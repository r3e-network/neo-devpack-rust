// Package main is the Neo N3 conformance oracle: it runs a NeoVM
// script through the neo-go embedded VM and reports the final stack,
// any notifications emitted via System.Runtime.Notify, and the final
// storage state.
//
// Two execution flavours share the same code path:
//
//   - PURE-COMPUTE contracts (arithmetic, control flow, ...) run on a
//     bare neo-go VM with no chain state.
//   - STATEFUL contracts (storage read/write, runtime queries,
//     witness checks, notifications) are serviced by an in-process
//     SyscallHandler (syscallEnv.handler) that mirrors neo-go's own
//     interop stack ABI but backs it with a single in-memory storage
//     map + request-seeded runtime values, instead of the full neo-go
//     Blockchain + InteropContext stack.
//
// The handler reimplements (rather than calls) neo-go's interop
// functions, because those take a *interop.Context bound to a real
// DAO/Blockchain. The pop/push order and stack-item types are mirrored
// 1:1 from the reference implementations in
// pkg/core/interop/storage/basic.go and pkg/core/interop/runtime/
// {engine,util,witness}.go of neo-go@v0.105.1 — see the per-syscall
// comments on syscallEnv.handler.
package main

import (
	"encoding/hex"
	"encoding/json"
	"errors"
	"flag"
	"fmt"
	"math/big"
	"os"
	"sort"
	"strconv"
	"strings"

	"github.com/nspcc-dev/neo-go/pkg/core/interop/interopnames"
	"github.com/nspcc-dev/neo-go/pkg/encoding/address"
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

	// Optional runtime environment knobs (all have sane defaults if
	// omitted, so existing requests keep working unchanged). Pointers
	// so that "field absent" is distinguishable from "field == 0".
	Time           *uint64 `json:"time,omitempty"`            // System.Runtime.GetTime (ms since epoch)
	Trigger        *int    `json:"trigger,omitempty"`         // System.Runtime.GetTrigger (default 0x40 Application)
	Network        *uint32 `json:"network,omitempty"`         // System.Runtime.GetNetwork (default mainnet magic)
	AddressVersion *int    `json:"address_version,omitempty"` // System.Runtime.GetAddressVersion (default NEO3 0x35)
	Random         *string `json:"random,omitempty"`          // System.Runtime.GetRandom (decimal; default 0)
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
		r := runLoaded(lc.nef, lc.man, &req)
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
	env := newSyscallEnv(&req, mfest)
	v.SyscallHandler = env.handler
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
	return runLoaded(nefFile, mfest, req)
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

// runLoaded executes one method on a pre-parsed contract with a fresh VM,
// servicing the common storage/runtime syscalls against in-memory state
// seeded from the request.
func runLoaded(nefFile *nef.File, mfest *manifest.Manifest, req *InvocationRequest) *InvocationResult {
	res := &InvocationResult{
		State:       "HALT",
		Events:      []Event{},
		StorageDiff: []Storage{},
	}

	// Build a fresh VM and install the in-process syscall handler so
	// contracts that touch storage / runtime run end-to-end. Pure-compute
	// contracts never reach the handler, so they are unaffected.
	v := vm.New()
	v.GasLimit = req.GasLimit
	if v.GasLimit <= 0 {
		v.GasLimit = 1_000_000_000
	}

	env := newSyscallEnv(req, mfest)
	v.SyscallHandler = env.handler

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
		// Even on FAULT, surface whatever events fired before the fault.
		res.Events = env.events
		return res
	}
	res.GasConsumed = v.GasConsumed()

	// Return stack.
	for i := 0; i < v.Estack().Len(); i++ {
		el := v.Estack().Peek(i)
		item := el.Item()
		res.ReturnStack = append(res.ReturnStack, formatItem(item))
	}

	// Final storage state + captured notifications.
	res.StorageDiff = env.storageDiff()
	res.Events = env.events

	return res
}

const callflagAll = 0x0F
const callflagNone = 0x00

// --------------------------------------------------------------------------
// In-process syscall handler.
//
// We install this as v.SyscallHandler so that contracts which issue
// System.Storage.* / System.Runtime.* / System.Contract.GetCallFlags
// syscalls run end-to-end on the bare neo-go VM, without the full
// Blockchain + interop.Context stack. The pop/push order and stack-item
// types below mirror neo-go@v0.105.1 exactly; the reference file + symbol
// is cited per syscall.
//
// A dummy storage context (any *storageContext InteropInterface) is shared
// by GetContext/GetReadOnlyContext/AsReadOnly. All keys map to one in-memory
// map, which is fine because these are single-contract tests.
// --------------------------------------------------------------------------

// storageContext is the InteropInterface pushed by System.Storage.GetContext.
// Mirrors storage.Context in pkg/core/interop/storage/basic.go (only the
// ReadOnly flag is meaningful here; there is a single contract / single map,
// so the contract ID is irrelevant).
type storageContext struct {
	ReadOnly bool
}

// syscallEnv holds the per-invocation mutable state backing the handler:
// the storage map, captured notifications, and request-derived runtime
// values (signers, time, trigger, ...).
type syscallEnv struct {
	// storage keyed by hex(key) -> {raw key bytes, raw value bytes}.
	store    map[string]kv
	events   []Event
	logs     []string
	manifest *manifest.Manifest

	signers        []util.Uint160 // signer account hashes (witness == true)
	time           uint64
	trigger        int
	network        uint32
	addressVersion int
	random         *big.Int
}

type kv struct {
	key []byte
	val []byte
}

func newSyscallEnv(req *InvocationRequest, mfest *manifest.Manifest) *syscallEnv {
	e := &syscallEnv{
		store:    map[string]kv{},
		events:   []Event{},
		manifest: mfest,
		// Defaults chosen to match a typical Application-trigger mainnet run.
		time:           0,
		trigger:        0x40, // trigger.Application (see trigger_type_string.go)
		network:        860833102, // N3 mainnet magic
		addressVersion: int(address.NEO3Prefix), // 0x35
		random:         big.NewInt(0),
	}

	// Seed in-memory storage from initial_storage. The `contract` field is
	// ignored (single contract / single map); key+value are hex.
	for _, s := range req.InitialStorage {
		key, err1 := hex.DecodeString(s.Key)
		val, err2 := hex.DecodeString(s.Value)
		if err1 != nil || err2 != nil {
			continue
		}
		e.store[hex.EncodeToString(key)] = kv{key: key, val: val}
	}

	// Signers: an account hex string is the 20-byte script hash (BE) that
	// CheckWitness should treat as "witnessed". Scopes are accepted but not
	// interpreted (any present scope grants the witness for that account),
	// which is the right behaviour for single-contract conformance tests.
	for _, s := range req.Signers {
		b, err := hex.DecodeString(s.Account)
		if err != nil {
			continue
		}
		h, err := util.Uint160DecodeBytesBE(b)
		if err != nil {
			continue
		}
		e.signers = append(e.signers, h)
	}

	// Optional runtime overrides.
	if req.Time != nil {
		e.time = *req.Time
	}
	if req.Trigger != nil {
		e.trigger = *req.Trigger
	}
	if req.Network != nil {
		e.network = *req.Network
	}
	if req.AddressVersion != nil {
		e.addressVersion = *req.AddressVersion
	}
	if req.Random != nil {
		n := new(big.Int)
		if _, ok := n.SetString(*req.Random, 10); ok {
			e.random = n
		}
	}
	return e
}

// storageDiff returns the final storage as a deterministically-ordered
// list of {key,value} hex pairs.
func (e *syscallEnv) storageDiff() []Storage {
	out := make([]Storage, 0, len(e.store))
	keys := make([]string, 0, len(e.store))
	for k := range e.store {
		keys = append(keys, k)
	}
	sort.Strings(keys)
	for _, k := range keys {
		kv := e.store[k]
		out = append(out, Storage{
			Key:   hex.EncodeToString(kv.key),
			Value: hex.EncodeToString(kv.val),
		})
	}
	return out
}

// Precomputed interop IDs. The id of a syscall is the little-endian
// uint32 of sha256(name)[:4] — exactly interopnames.ToID. (Note:
// vm.GetInteropID reads those 4 bytes back out of the SYSCALL operand,
// so ToID is the right direction here, NOT GetInteropID on the name.)
// Computed once at init from the canonical names so the mapping can
// never drift from neo-go's.
var (
	idStorageGetContext         = sysID("System.Storage.GetContext")
	idStorageGetReadOnlyContext = sysID("System.Storage.GetReadOnlyContext")
	idStorageAsReadOnly         = sysID("System.Storage.AsReadOnly")
	idStorageGet                = sysID("System.Storage.Get")
	idStoragePut                = sysID("System.Storage.Put")
	idStorageDelete             = sysID("System.Storage.Delete")

	idRuntimeLog                 = sysID("System.Runtime.Log")
	idRuntimeNotify              = sysID("System.Runtime.Notify")
	idRuntimeCheckWitness        = sysID("System.Runtime.CheckWitness")
	idRuntimeGetTime             = sysID("System.Runtime.GetTime")
	idRuntimeGetTrigger          = sysID("System.Runtime.GetTrigger")
	idRuntimePlatform            = sysID("System.Runtime.Platform")
	idRuntimeGetNetwork          = sysID("System.Runtime.GetNetwork")
	idRuntimeGetAddressVersion   = sysID("System.Runtime.GetAddressVersion")
	idRuntimeGetInvocationCount  = sysID("System.Runtime.GetInvocationCounter")
	idRuntimeGetRandom           = sysID("System.Runtime.GetRandom")
	idRuntimeGetScriptContainer  = sysID("System.Runtime.GetScriptContainer")
	idRuntimeGasLeft             = sysID("System.Runtime.GasLeft")
	idRuntimeGetExecutingHash    = sysID("System.Runtime.GetExecutingScriptHash")
	idRuntimeGetCallingHash      = sysID("System.Runtime.GetCallingScriptHash")
	idRuntimeGetEntryHash        = sysID("System.Runtime.GetEntryScriptHash")

	idContractGetCallFlags = sysID("System.Contract.GetCallFlags")
)

func sysID(name string) uint32 {
	return interopnames.ToID([]byte(name))
}

// handler is the v.SyscallHandler. It dispatches on the interop id and
// services the syscall directly against the VM's estack + the in-memory
// env state. Any unsupported syscall returns an error, which neo-go turns
// into a VM FAULT (matching the previous bare-VM behaviour for those).
func (e *syscallEnv) handler(v *vm.VM, id uint32) error {
	switch id {

	// ---- Storage -----------------------------------------------------
	// storage.GetContext / GetReadOnlyContext: pushes a Context
	// InteropInterface. Ref: storage/basic.go getContextInternal.
	case idStorageGetContext:
		v.Estack().PushItem(stackitem.NewInterop(&storageContext{ReadOnly: false}))
		return nil
	case idStorageGetReadOnlyContext:
		v.Estack().PushItem(stackitem.NewInterop(&storageContext{ReadOnly: true}))
		return nil

	// storage.ContextAsReadOnly: pops context, pushes a read-only context.
	// Ref: storage/basic.go ContextAsReadOnly.
	case idStorageAsReadOnly:
		stc, err := popStorageContext(v)
		if err != nil {
			return err
		}
		if !stc.ReadOnly {
			stc = &storageContext{ReadOnly: true}
		}
		v.Estack().PushItem(stackitem.NewInterop(stc))
		return nil

	// storage.Get: pops (context, key), pushes ByteString or Null.
	// Ref: storage/basic.go Get.
	case idStorageGet:
		if _, err := popStorageContext(v); err != nil {
			return err
		}
		key := v.Estack().Pop().Bytes()
		if rec, ok := e.store[hex.EncodeToString(key)]; ok {
			// A stored empty value (e.g. integer 0 -> zero-length bytes) must be
			// a non-nil ByteString: the real C# VM converts empty->Integer as 0,
			// and neo-go's FromBytes rejects a *nil* slice. Keeping it non-nil
			// mirrors a faithful storage layer and avoids a spurious FAULT.
			bs := rec.val
			if bs == nil {
				bs = []byte{}
			}
			v.Estack().PushItem(stackitem.NewByteArray(bs))
		} else {
			v.Estack().PushItem(stackitem.Null{})
		}
		return nil

	// storage.Put: pops (context, key, value); writes. Ref: storage/basic.go Put.
	case idStoragePut:
		stc, err := popStorageContext(v)
		if err != nil {
			return err
		}
		if stc.ReadOnly {
			return errors.New("storage.Context is read only")
		}
		key := v.Estack().Pop().Bytes()
		val := v.Estack().Pop().Bytes()
		// Copy out of the VM-owned slices.
		k := append([]byte(nil), key...)
		val = append([]byte(nil), val...)
		e.store[hex.EncodeToString(k)] = kv{key: k, val: val}
		return nil

	// storage.Delete: pops (context, key); deletes. Ref: storage/basic.go Delete.
	case idStorageDelete:
		stc, err := popStorageContext(v)
		if err != nil {
			return err
		}
		if stc.ReadOnly {
			return errors.New("storage.Context is read only")
		}
		key := v.Estack().Pop().Bytes()
		delete(e.store, hex.EncodeToString(key))
		return nil

	// ---- Runtime -----------------------------------------------------
	// runtime.Log: pops a message string. Ref: runtime/engine.go Log.
	case idRuntimeLog:
		msg := v.Estack().Pop().String()
		e.logs = append(e.logs, msg)
		return nil

	// runtime.Notify: pops (name, state-array); records an Event.
	// Ref: runtime/engine.go Notify (pops name first, then the elem array).
	case idRuntimeNotify:
		name := v.Estack().Pop().String()
		elem := v.Estack().Pop()
		args := elem.Array()
		state := make([]string, 0, len(args))
		for _, a := range args {
			state = append(state, formatItem(a))
		}
		e.events = append(e.events, Event{Name: name, State: state})
		return nil

	// runtime.CheckWitness: pops hash-or-pubkey bytes, pushes Bool.
	// Ref: runtime/witness.go CheckWitness — a 20-byte arg is a script
	// hash; we treat it as witnessed iff it is among the request signers.
	// (33-byte pubkeys are not resolved to a script hash here; they yield
	// false unless their bytes happen to match a signer, which is the
	// conservative/safe answer for single-contract conformance tests.)
	case idRuntimeCheckWitness:
		raw := v.Estack().Pop().Bytes()
		res := false
		if h, err := util.Uint160DecodeBytesBE(raw); err == nil {
			for _, s := range e.signers {
				if s.Equals(h) {
					res = true
					break
				}
			}
		}
		v.Estack().PushItem(stackitem.Bool(res))
		return nil

	// runtime.GetTime: pushes uint64 timestamp. Ref: runtime/engine.go GetTime.
	case idRuntimeGetTime:
		v.Estack().PushItem(stackitem.NewBigInteger(new(big.Int).SetUint64(e.time)))
		return nil

	// runtime.GetTrigger: pushes the trigger byte. Ref: runtime/engine.go GetTrigger.
	case idRuntimeGetTrigger:
		v.Estack().PushItem(stackitem.NewBigInteger(big.NewInt(int64(e.trigger))))
		return nil

	// runtime.Platform: pushes "NEO". Ref: runtime/engine.go Platform.
	case idRuntimePlatform:
		v.Estack().PushItem(stackitem.NewByteArray([]byte("NEO")))
		return nil

	// runtime.GetNetwork: pushes magic. Ref: runtime/util.go GetNetwork.
	case idRuntimeGetNetwork:
		v.Estack().PushItem(stackitem.NewBigInteger(big.NewInt(int64(e.network))))
		return nil

	// runtime.GetAddressVersion: pushes address prefix. Ref: runtime/util.go GetAddressVersion.
	case idRuntimeGetAddressVersion:
		v.Estack().PushItem(stackitem.NewBigInteger(big.NewInt(int64(e.addressVersion))))
		return nil

	// runtime.GetInvocationCounter: single contract -> always 1.
	// Ref: runtime/util.go GetInvocationCounter (initialises to 1).
	case idRuntimeGetInvocationCount:
		v.Estack().PushItem(stackitem.NewBigInteger(big.NewInt(1)))
		return nil

	// runtime.GetRandom: pushes a deterministic (request-seeded) value.
	// Ref: runtime/util.go GetRandom (we don't reproduce the murmur128
	// chain; a fixed seedable value is enough for conformance).
	case idRuntimeGetRandom:
		v.Estack().PushItem(stackitem.NewBigInteger(new(big.Int).Set(e.random)))
		return nil

	// runtime.GetScriptContainer: benign default (Null). Ref:
	// runtime/engine.go GetScriptContainer pushes the tx/block item; with
	// no container we push Null, like CurrentSigners' no-tx fallback.
	case idRuntimeGetScriptContainer:
		v.Estack().PushItem(stackitem.Null{})
		return nil

	// runtime.GasLeft: pushes remaining gas. Ref: runtime/util.go GasLeft.
	case idRuntimeGasLeft:
		if v.GasLimit == -1 {
			v.Estack().PushItem(stackitem.NewBigInteger(big.NewInt(v.GasLimit)))
		} else {
			v.Estack().PushItem(stackitem.NewBigInteger(big.NewInt(v.GasLimit - v.GasConsumed())))
		}
		return nil

	// Script-hash queries: defer to the VM's own context tracking.
	// Ref: runtime/engine.go GetExecuting/Calling/EntryScriptHash.
	case idRuntimeGetExecutingHash:
		return v.PushContextScriptHash(0)
	case idRuntimeGetCallingHash:
		h := v.GetCallingScriptHash()
		v.Estack().PushItem(stackitem.NewByteArray(h.BytesBE()))
		return nil
	case idRuntimeGetEntryHash:
		return v.PushContextScriptHash(len(v.Istack()) - 1)

	// ---- Contract ----------------------------------------------------
	// contract.GetCallFlags: pushes current context call flags.
	// Ref: contract/call.go GetCallFlags.
	case idContractGetCallFlags:
		v.Estack().PushItem(stackitem.NewBigInteger(big.NewInt(int64(v.Context().GetCallFlags()))))
		return nil
	}

	// Unsupported syscall: surface as FAULT with a clear message rather
	// than silently pushing a wrong value. Deliberately unsupported:
	//   - System.Storage.Find (iterator protocol; see top-of-file note)
	//   - cross-contract System.Contract.Call, crypto, etc.
	if name, err := interopnames.FromID(id); err == nil {
		return fmt.Errorf("unsupported syscall %s (id=%d)", name, id)
	}
	return fmt.Errorf("unsupported syscall id=%d", id)
}

// popStorageContext pops the top estack item and asserts it is a
// *storageContext (the InteropInterface produced by GetContext et al).
// Mirrors the `stc, ok := ...(*Context)` guard in storage/basic.go.
func popStorageContext(v *vm.VM) (*storageContext, error) {
	val := v.Estack().Pop().Value()
	stc, ok := val.(*storageContext)
	if !ok {
		return nil, fmt.Errorf("%T is not a storage.Context", val)
	}
	return stc, nil
}

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
