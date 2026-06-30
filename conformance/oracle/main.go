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
	"strconv"
	"strings"

	"github.com/nspcc-dev/neo-go/pkg/smartcontract"
	"github.com/nspcc-dev/neo-go/pkg/smartcontract/manifest"
	"github.com/nspcc-dev/neo-go/pkg/smartcontract/nef"
	"github.com/nspcc-dev/neo-go/pkg/util"
	"github.com/nspcc-dev/neo-go/pkg/vm"
	"github.com/nspcc-dev/neo-go/pkg/vm/stackitem"
	"github.com/nspcc-dev/neo-go/pkg/vm/vmstate"
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
	batch := flag.Bool("batch", false, "Batch mode: -in is JSONL of InvocationRequests; contracts are parsed once and cached by path; -out is JSONL of {st,top}")
	disasm := flag.String("disasm", "", "Disassemble: path to a NEF file")
	from := flag.Int("from", 0, "disasm/trace start IP filter")
	count := flag.Int("count", 80, "disasm instruction count")
	trace := flag.Bool("trace", false, "Trace mode: step -in request, print each instr + estack (filter IP >= -from, <= -to)")
	to := flag.Int("to", 1<<30, "trace IP filter upper bound")
	flag.Parse()

	if *disasm != "" {
		runDisasm(*disasm, *from, *count)
		return
	}

	if *trace {
		runTrace(*inPath, *from, *to)
		return
	}

	if *inPath == "" {
		fmt.Fprintln(os.Stderr, "usage: neo-n3-oracle -in <request.json> [-out <result.json>] [-batch] | -disasm <nef> [-from N] [-count N]")
		os.Exit(2)
	}

	if *batch {
		runBatch(*inPath, *outPath)
		return
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

// runBatch parses each contract once (cached by nef|manifest path) and runs
// every InvocationRequest line, emitting a compact {st,top} JSONL result in
// input order. This avoids the per-call process-startup cost when fuzzing tens
// of thousands of invocations against the same contract.
func runBatch(inPath, outPath string) {
	data, err := os.ReadFile(inPath)
	if err != nil {
		fmt.Fprintln(os.Stderr, "batch read:", err)
		os.Exit(2)
	}
	type loaded struct {
		nef *nef.File
		man *manifest.Manifest
	}
	cache := map[string]*loaded{}
	var buf []byte
	for _, line := range strings.Split(strings.TrimSpace(string(data)), "\n") {
		line = strings.TrimSpace(line)
		if line == "" {
			continue
		}
		var req InvocationRequest
		if err := json.Unmarshal([]byte(line), &req); err != nil {
			buf = append(buf, []byte("{\"st\":\"FAULT\",\"top\":null}\n")...)
			continue
		}
		key := req.NEFPath + "|" + req.ManifestPath
		lc := cache[key]
		if lc == nil {
			nf, mf, ferr := loadContract(req.NEFPath, req.ManifestPath)
			if ferr != nil {
				buf = append(buf, []byte("{\"st\":\"FAULT\",\"top\":null}\n")...)
				continue
			}
			lc = &loaded{nf, mf}
			cache[key] = lc
		}
		r := runLoaded(lc.nef, lc.man, req.Method, req.Arguments, req.GasLimit)
		top := "null"
		if len(r.ReturnStack) > 0 {
			top = strconv.Quote(r.ReturnStack[0])
		}
		buf = append(buf, []byte(fmt.Sprintf("{\"st\":%q,\"top\":%s}\n", r.State, top))...)
	}
	if outPath != "" {
		_ = os.WriteFile(outPath, buf, 0o644)
	} else {
		fmt.Print(string(buf))
	}
}

// runTrace single-steps the VM for one request, printing each instruction in
// the IP window [from,to] with the post-step estack (top few items).
func runTrace(inPath string, from, to int) {
	reqBytes, err := os.ReadFile(inPath)
	if err != nil {
		fmt.Fprintln(os.Stderr, "read request:", err)
		os.Exit(2)
	}
	var req InvocationRequest
	if err := json.Unmarshal(reqBytes, &req); err != nil {
		fmt.Fprintln(os.Stderr, "parse request:", err)
		os.Exit(2)
	}
	nefFile, mfest, ferr := loadContract(req.NEFPath, req.ManifestPath)
	if ferr != nil {
		fmt.Fprintln(os.Stderr, ferr)
		os.Exit(2)
	}
	var methodOff int
	hasReturn := true
	for _, m := range mfest.ABI.Methods {
		if m.Name == req.Method {
			methodOff = int(m.Offset)
			hasReturn = m.ReturnType != smartcontract.VoidType
			break
		}
	}
	v := vm.New()
	v.GasLimit = 2_000_000_000
	v.LoadNEFMethod(nefFile, util.Uint160{}, util.Uint160{}, callflagAll, hasReturn, methodOff, -1, nil)
	for i := len(req.Arguments) - 1; i >= 0; i-- {
		item, _ := parseArgument(req.Arguments[i])
		v.Estack().PushItem(item)
	}
	steps := 0
	for !v.HasStopped() && steps < 200000 {
		ip, op := v.Context().CurrInstr()
		show := ip >= from && ip <= to
		if err := v.Step(); err != nil {
			fmt.Printf("%5d: %-12s STEP-ERR %v\n", ip, op.String(), err)
			break
		}
		if show {
			var tops []string
			n := v.Estack().Len()
			for i := 0; i < n && i < 4; i++ {
				tops = append(tops, formatItem(v.Estack().Peek(i).Item()))
			}
			fmt.Printf("%5d: %-12s estack=[%s]\n", ip, op.String(), joinComma(tops))
		}
		steps++
	}
	fmt.Printf("--- final state=%v steps=%d ---\n", v.State() == vmstate.Halt, steps)
}

// runDisasm prints `IP: OPCODE param` for a NEF's script, starting at -from.
func runDisasm(nefPath string, from, count int) {
	nefBytes, err := os.ReadFile(nefPath)
	if err != nil {
		fmt.Fprintln(os.Stderr, "read NEF:", err)
		os.Exit(2)
	}
	nefFile, err := nef.FileFromBytes(nefBytes)
	if err != nil {
		fmt.Fprintln(os.Stderr, "parse NEF:", err)
		os.Exit(2)
	}
	script := nefFile.Script
	ctx := vm.NewContext(script)
	if from > 0 && from < len(script) {
		ctx.Jump(from)
	}
	for i := 0; i < count; i++ {
		if ctx.NextIP() >= len(script) {
			break
		}
		op, param, err := ctx.Next()
		ip := ctx.IP()
		if err != nil {
			fmt.Printf("%5d: %-12s ERR %v\n", ip, op.String(), err)
			break
		}
		if len(param) > 0 {
			fmt.Printf("%5d: %-12s %x\n", ip, op.String(), param)
		} else {
			fmt.Printf("%5d: %s\n", ip, op.String())
		}
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
	nefFile, mfest, ferr := loadContract(req.NEFPath, req.ManifestPath)
	if ferr != nil {
		return &InvocationResult{State: "FAULT", ErrorMessage: ferr.Error()}
	}
	return runLoaded(nefFile, mfest, req.Method, req.Arguments, req.GasLimit)
}

// loadContract parses a NEF + manifest from disk once (cacheable for batch mode).
func loadContract(nefPath, manifestPath string) (*nef.File, *manifest.Manifest, error) {
	nefBytes, err := os.ReadFile(nefPath)
	if err != nil {
		return nil, nil, fmt.Errorf("read NEF: %v", err)
	}
	nefFile, err := nef.FileFromBytes(nefBytes)
	if err != nil {
		return nil, nil, fmt.Errorf("parse NEF: %v", err)
	}
	manifestBytes, err := os.ReadFile(manifestPath)
	if err != nil {
		return nil, nil, fmt.Errorf("read manifest: %v", err)
	}
	var mfest manifest.Manifest
	if err := json.Unmarshal(manifestBytes, &mfest); err != nil {
		return nil, nil, fmt.Errorf("parse manifest: %v", err)
	}
	return &nefFile, &mfest, nil
}

// runLoaded executes one method on a pre-parsed contract with a fresh VM.
func runLoaded(nefFile *nef.File, mfest *manifest.Manifest, method string, arguments []Argument, gasLimit int64) *InvocationResult {
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
	v.GasLimit = gasLimit
	if v.GasLimit <= 0 {
		v.GasLimit = 1_000_000_000
	}

	// Find the method offset in the manifest. We use
	// LoadNEFMethod so the VM starts at the method's body
	// (skipping the entry stub and _initialize).
	var methodOff int
	var hasReturn bool = true
	for _, m := range mfest.ABI.Methods {
		if m.Name == method {
			methodOff = int(m.Offset)
			// ParamType.VoidType = no return.
			hasReturn = m.ReturnType != smartcontract.VoidType
			break
		}
	}
	if methodOff == 0 && method != "" {
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
		nefFile,
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
	for i := len(arguments) - 1; i >= 0; i-- {
		item, err := parseArgument(arguments[i])
		if err != nil {
			return &InvocationResult{State: "FAULT", ErrorMessage: fmt.Sprintf("parse arg %q: %v", arguments[i].Value, err)}
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
		// NB: stackitem.BigInteger.String() returns the type name, not the
		// value; the numeric value lives in Value() (*big.Int).
		return fmt.Sprintf("%v", it.Value())
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
