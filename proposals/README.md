# Proposals

Task-focused proposals for significant changes to the codebase.

## Purpose

This directory contains formal proposals for:
- Major refactorings or architectural changes
- New features requiring cross-cutting changes
- Complex bug fixes that need design discussion

## File Naming

Proposals use numbered prefixes to indicate dependencies:
- `00_name.md` - Standalone work, no blockers
- `01_name.md` - Sequential work (must complete before dependent work)
- `02a_name.md, 02b_name.md` - Parallel work (can run simultaneously)
- `NN_name.proposal.md` - Larger proposals with detailed implementation plans

## Bug Reports

The `bug_reports/` subdirectory tracks known issues requiring investigation or complex fixes.

## Completed Work

When proposals are completed, they are removed from this directory. The git history preserves the proposal details if needed for reference.
