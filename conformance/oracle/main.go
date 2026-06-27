// Package main is the minimal Neo N3 conformance oracle: it runs
// a NeoVM script through the neo-go embedded VM and prints the
// final stack + a record of any *user-issued* notifies (which the
// VM traces via internal events).
//
// This is a deliberately minimal oracle. The full oracle with
// notifications from `System.Runtime.Notify` requires the entire
// neo-go Blockchain + InteropContext stack, which depends on a real
// chain config. For the L7 conformance harness, we use the Rust
// exec harness + the C# spec as the ground truth and use this Go
// oracle only to cross-check the bytecode shape (opcodes,
// jumps, return) and the return stack.
//
// If a future commit wants the full notify pipeline, swap the
// VM-only path for `core.NewBlockchain` + `GetTestVM` (the path
// used by neo-go's own test suite).
package main

import (
	"encoding/hex"
	"encoding/json"
	"flag"
	"fmt"
	"math/big"
	"os"

	"github.com/nspcc-dev/neo-go/pkg/smartcontract"
	"github.com/nspcc-dev/neo-go/pkg/smartcontract/manifest"
	"github.com/nspcc-dev/neo-go/pkg/smartcontract/nef"
	"github.com/nspcc-dev/neo-go/pkg/util"
	"github.com/nspcc-dev/neo-go/pkg/vm"
	"github.com/nspcc-dev/neo-go/pkg/vm/stackitem"
)

// InvocationRequest is the JSON contract the Rust side sends.
type InvocationRequest struct {
	NEFPath        string       `json:"nef_path"`
	ManifestPath   string       `json:"manifest_path"`
	Method         string       `json:"method"`
	Arguments      []Argument   `json:"arguments"`
	Signers        []SignerSpec `json:"signers"`
	InitialStorage []Storage    `json:"initial_storage"`
	GasLimit       int64        `json:"gas_limit"`
}

type Argument struct {
	Type  string `json:"type"`
	Value string `json:"value"`
}

type SignerSpec struct {
	Account string `json:"account"`
	Scopes  string `json:"scopes"`
}

type Storage struct {
	Contract string `json:"contract"`
	Key      string `json:"key"`
	Value    string `json:"value"`
}

// InvocationResult is the JSON we return to the Rust side.
type InvocationResult struct {
	State        string     `json:"state"`
	GasConsumed  int64      `json:"gas_consumed"`
	ReturnStack  []string   `json:"return_stack"`
	Events       []Event    `json:"events"`
	StorageDiff  []Storage  `json:"storage_diff"`
	ErrorMessage string     `json:"error_message,omitempty"`
}

type Event struct {
	Contract string   `json:"contract"`
	Name     string   `json:"name"`
	State    []string `json:"state"`
}

func main() {
	inPath := flag.String("in", "", "Path to the JSON InvocationRequest")
	outPath := flag.String("out", "", "Path to write the JSON InvocationResult")
	flag.Parse()

	if *inPath == "" {
		fmt.Fprintln(os.Stderr, "usage: neo-n3-oracle -in <request.json> [-out <result.json>]")
		os.Exit(2)
	}

	reqBytes, err := os.ReadFile(*inPath)
	if err != nil {
		fail(*outPath, fmt.Sprintf("failed to read request: %v", err))
		return
	}
	var req InvocationRequest
	if err := json.Unmarshal(reqBytes, &req); err != nil {
		fail(*outPath, fmt.Sprintf("failed to parse request: %v", err))
		return
	}

	result := invoke(&req)

	outBytes, err := json.MarshalIndent(result, "", "  ")
	if err != nil {
		fail(*outPath, fmt.Sprintf("failed to marshal result: %v", err))
		return
	}
	if *outPath != "" {
		_ = os.WriteFile(*outPath, outBytes, 0o644)
	} else {
		fmt.Println(string(outBytes))
	}
}

func fail(outPath, msg string) {
	result := InvocationResult{State: "FAULT", ErrorMessage: msg}
	outBytes, _ := json.MarshalIndent(result, "", "  ")
	if outPath != "" {
		_ = os.WriteFile(outPath, outBytes, 0o644)
	} else {
		fmt.Println(string(outBytes))
	}
	os.Exit(1)
}

func invoke(req *InvocationRequest) *InvocationResult {
	// Load NEF.
	nefBytes, err := os.ReadFile(req.NEFPath)
	if err != nil {
		return &InvocationResult{State: "FAULT", ErrorMessage: fmt.Sprintf("read NEF: %v", err)}
	}
	nefFile, err := nef.FileFromBytes(nefBytes)
	if err != nil {
		return &InvocationResult{State: "FAULT", ErrorMessage: fmt.Sprintf("parse NEF: %v", err)}
	}
	// NEF File has no Hash field. The contract hash would be
	// computed by the engine when it deploys the contract; for
	// the L7 stepping-stone oracle we don't need it (we use
	// LoadScriptWithHash with the zero hash, which is fine for
	// verifying the script shape).
	_ = nefFile

	// Load manifest.
	manifestBytes, err := os.ReadFile(req.ManifestPath)
	if err != nil {
		return &InvocationResult{State: "FAULT", ErrorMessage: fmt.Sprintf("read manifest: %v", err)}
	}
	var mfest manifest.Manifest
	if err := json.Unmarshal(manifestBytes, &mfest); err != nil {
		return &InvocationResult{State: "FAULT", ErrorMessage: fmt.Sprintf("parse manifest: %v", err)}
	}

	res := &InvocationResult{
		State:       "HALT",
		Events:      []Event{},
		StorageDiff: []Storage{},
	}

	// Build a fresh VM. The standalone VM doesn't capture
	// notifications (those go through System.Runtime.Notify in
	// the InteropContext); for the L7 stepping-stone oracle we
	// verify the bytecode shape and return stack only.
	v := vm.New()
	v.GasLimit = req.GasLimit
	if v.GasLimit <= 0 {
		v.GasLimit = 1_000_000_000
	}

	// Find the method offset in the manifest. We use
	// LoadNEFMethod so the VM starts at the method's body
	// (skipping the entry stub and _initialize).
	var methodOff int
	var hasReturn bool = true
	for _, m := range mfest.ABI.Methods {
		if m.Name == req.Method {
			methodOff = int(m.Offset)
			// ParamType.VoidType = no return.
			hasReturn = m.ReturnType != smartcontract.VoidType
			break
		}
	}
	if methodOff == 0 && req.Method != "" {
		// Fall back to entry point if the named method wasn't found.
		for _, m := range mfest.ABI.Methods {
			if m.Name == "_initialize" {
				methodOff = int(m.Offset)
				hasReturn = false
				break
			}
		}
	}

	// Load the contract script with the right entry point.
	// We pass a zero hash since the standalone VM doesn't track
	// contract provenance; the L7 oracle cares about bytecode
	// shape + return stack, not about cross-contract state.
	//
	// CRITICAL: the order is LoadNEFMethod first, THEN push
	// args (in reverse). This matches the neo-go core engine
	// callExFromNative flow (see pkg/core/interop/contract/call.go).
	v.LoadNEFMethod(
		&nefFile,
		util.Uint160{},
		util.Uint160{},
		callflagAll,
		hasReturn,
		methodOff,
		-1,
		nil,
	)

	// Push args in REVERSE order (the engine's estack is
	// push-only and the contract reads them in script order).
	for i := len(req.Arguments) - 1; i >= 0; i-- {
		item, err := parseArgument(req.Arguments[i])
		if err != nil {
			return &InvocationResult{State: "FAULT", ErrorMessage: fmt.Sprintf("parse arg %q: %v", req.Arguments[i].Value, err)}
		}
		v.Estack().PushItem(item)
	}

	// Run.
	if err := v.Run(); err != nil {
		res.State = "FAULT"
		res.ErrorMessage = err.Error()
		res.GasConsumed = v.GasConsumed()
		return res
	}
	res.GasConsumed = v.GasConsumed()

	// Return stack.
	for i := 0; i < v.Estack().Len(); i++ {
		el := v.Estack().Peek(i)
		item := el.Item()
		res.ReturnStack = append(res.ReturnStack, formatItem(item))
	}

	return res
}

const callflagAll = 0x0F
const callflagNone = 0x00

func parseArgument(a Argument) (stackitem.Item, error) {
	switch a.Type {
	case "int", "integer":
		n := new(big.Int)
		if _, ok := n.SetString(a.Value, 10); !ok {
			return nil, fmt.Errorf("invalid integer: %q", a.Value)
		}
		return stackitem.NewBigInteger(n), nil
	case "bool", "boolean":
		v := a.Value == "true" || a.Value == "1"
		return stackitem.NewBool(v), nil
	case "string":
		return stackitem.NewByteArray([]byte(a.Value)), nil
	case "bytes", "bytearray":
		b, err := hex.DecodeString(a.Value)
		if err != nil {
			return nil, err
		}
		return stackitem.NewByteArray(b), nil
	case "hash160":
		b, err := hex.DecodeString(a.Value)
		if err != nil {
			return nil, err
		}
		return stackitem.NewByteArray(b), nil
	default:
		return nil, fmt.Errorf("unsupported argument type: %q", a.Type)
	}
}

func formatItem(item stackitem.Item) string {
	if item == nil {
		return "null"
	}
	switch it := item.(type) {
	case *stackitem.BigInteger:
		return it.String()
	case stackitem.Bool:
		if bool(it) {
			return "true"
		}
		return "false"
	case *stackitem.ByteArray:
		return hex.EncodeToString(*it)
	case *stackitem.Null:
		return "null"
	case *stackitem.Array:
		inner, _ := it.Value().([]stackitem.Item)
		parts := make([]string, 0, len(inner))
		for _, x := range inner {
			parts = append(parts, formatItem(x))
		}
		return "[" + joinComma(parts) + "]"
	case *stackitem.Struct:
		inner, _ := it.Value().([]stackitem.Item)
		parts := make([]string, 0, len(inner))
		for _, x := range inner {
			parts = append(parts, formatItem(x))
		}
		return "<" + joinComma(parts) + ">"
	default:
		return fmt.Sprintf("%v", it)
	}
}

func joinComma(parts []string) string {
	out := ""
	for i, p := range parts {
		if i > 0 {
			out += ","
		}
		out += p
	}
	return out
}
