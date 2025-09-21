# codebuddy Roadmap

This document outlines the future direction and planned features for codebuddy.

## Vision

Make codebuddy the go-to MCP server for Language Server Protocol integration, enabling AI assistants to understand and navigate codebases as effectively as human developers.

## Current Status (v1.x)

- ✅ Core LSP functionality (go to definition, find references, rename symbol)
- ✅ Multi-language support via configurable LSP servers  
- ✅ TypeScript/JavaScript support out of the box
- ✅ Smart CLI with auto-detection (`init`, `status`, `fix`, `config`, `logs`)
- ✅ Auto-installation of missing language servers
- ✅ Real-time server availability checking
- ✅ Comprehensive error handling and logging

## Short-term Goals (Next 3 months)

### v1.1 - Enhanced Language Support  
- ✅ Auto-detection of installed language servers (`codebuddy init`)
- ✅ Built-in configurations for 15+ programming languages
- [ ] Language-specific initialization options
- ✅ Better error messages for missing language servers

### v1.2 - Performance Improvements
- [ ] Connection pooling for LSP servers
- [ ] Lazy loading of language servers
- [ ] Caching of symbol information
- [ ] Parallel request handling

### v1.3 - Developer Experience
- ✅ Interactive configuration generator (`codebuddy init`)
- ✅ Debugging mode with detailed logs (`codebuddy logs`)
- ✅ Health check command (`codebuddy status`)
- ✅ Integration test suite for each language

## Medium-term Goals (6-12 months)

### v2.0 - Advanced LSP Features
- ✅ Code completion support (`get_completions`)
- ✅ Hover information (`get_hover`) 
- ✅ Signature help (`get_signature_help`)
- ✅ Document symbols (`get_document_symbols`)
- ✅ Workspace symbols (`search_workspace_symbols`)

### v2.1 - Project Intelligence
- ✅ Project-wide symbol search (`search_workspace_symbols`)
- ✅ Call hierarchy navigation (`prepare_call_hierarchy`, `get_call_hierarchy_*`)
- ✅ Type hierarchy support (`prepare_type_hierarchy`, `get_type_hierarchy_*`)
- [ ] Import/dependency analysis

### v2.2 - Integration Ecosystem
- [ ] Plugin system for custom tools
- [ ] Integration with popular IDEs
- [ ] Docker support for isolated environments
- [ ] Cloud-hosted LSP server option

## Long-term Vision (1+ years)

### Semantic Code Understanding
- [ ] Cross-language reference tracking
- [ ] Semantic diff analysis
- [ ] Code pattern recognition
- [ ] Refactoring suggestions

### AI-Enhanced Features
- [ ] Natural language to symbol mapping
- [ ] Context-aware code navigation
- [ ] Intelligent code summarization
- [ ] Automated documentation generation

### Enterprise Features
- [ ] Multi-repository support
- [ ] Access control and security policies
- [ ] Audit logging
- [ ] Performance analytics

## Community Driven Features

We're open to community suggestions! Features requested by users:
- [ ] Support for notebooks (Jupyter, Observable)
- [ ] GraphQL schema navigation
- [ ] Database schema integration
- [ ] API documentation linking

## How to Contribute

1. **Vote on features**: Use 👍 reactions on issues to show interest
2. **Propose new features**: Open a feature request issue
3. **Implement features**: Check issues labeled "help wanted"
4. **Add language support**: See CONTRIBUTING.md

## Release Schedule

- **Patch releases**: As needed for bug fixes
- **Minor releases**: Monthly with new features
- **Major releases**: Annually with breaking changes

## Success Metrics

- Number of supported languages
- Response time for LSP operations
- Community contributions
- User satisfaction (GitHub stars, npm downloads)

---

This roadmap is a living document and will be updated based on community feedback and project evolution.