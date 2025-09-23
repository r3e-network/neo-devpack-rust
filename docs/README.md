# Neo LLVM Documentation

This directory contains the essential design, architecture, and technical documentation for the Neo LLVM project.

## 📋 **Documentation Index**

### 🏗️ **Architecture & Design**
- **[neo-llvm-roadmap.md](neo-llvm-roadmap.md)** - Project roadmap, goals, and overall architecture
- **[implementation-plan.md](implementation-plan.md)** - Detailed implementation plan and technical architecture
- **[neo-n3-backend.md](neo-n3-backend.md)** - Neo N3 LLVM backend architecture and design

### 🔧 **Technical Specifications**
- **[llvm-to-neovm-translation.md](llvm-to-neovm-translation.md)** - Technical explanation of register-based to stack-based translation
- **[nef-format-specification.md](nef-format-specification.md)** - Neo Executable Format (NEF) specification
- **[complete-neon3-support.md](complete-neon3-support.md)** - Complete Neo N3 opcode and syscall support documentation

### 🦀 **Rust Integration**
- **[rust-framework.md](rust-framework.md)** - Rust development framework design and architecture
- **[rust-integration.md](rust-integration.md)** - Rust integration with LLVM backend

## 🎯 **Documentation Purpose**

These documents provide:
- **Design Decisions**: Why certain architectural choices were made
- **Technical Specifications**: How the system works internally
- **Implementation Guidance**: How to extend and maintain the system
- **Integration Details**: How different components work together

## 📖 **Reading Order**

For new contributors, we recommend reading in this order:
1. Start with `neo-llvm-roadmap.md` for project overview
2. Read `implementation-plan.md` for technical architecture
3. Review `llvm-to-neovm-translation.md` for core translation concepts
4. Explore `rust-framework.md` and `rust-integration.md` for Rust development
5. Reference `nef-format-specification.md` and `complete-neon3-support.md` as needed

## 🔄 **Documentation Maintenance**

These documents are maintained as the authoritative source of truth for:
- System architecture and design decisions
- Technical specifications and interfaces
- Implementation guidelines and best practices
- Integration patterns and workflows

All code changes should align with the specifications in these documents.
