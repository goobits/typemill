# Project A - Setup Notes

## Project Information

- **Repository**: [URL here]
- **Language(s)**: [e.g., Rust, TypeScript]
- **Description**: [Brief description]
- **Date Started**: YYYY-MM-DD

## Setup Steps

### System Dependencies

```bash
# List any apt-get install commands needed
```

### Services

```bash
# Start services
docker-compose -f docker-compose.mill-test.yml up -d

# Verify services
docker-compose -f docker-compose.mill-test.yml ps
```

### Mill Configuration

```bash
cd repo
mill setup
mill status
```

### Baseline Test

```bash
# Commands to verify project works before refactoring
```

## Refactoring Tests

### Test 1: [Operation Name]

**Date**: YYYY-MM-DD

**Command**:
```bash
mill tool rename ...
```

**Result**: ✅ Success / ❌ Failure

**Notes**:
- [Observations]
- [Edge cases found]
- [Issues discovered]

### Test 2: [Operation Name]

**Date**: YYYY-MM-DD

**Command**:
```bash
mill tool move ...
```

**Result**: ✅ Success / ❌ Failure

**Notes**:
- [Observations]

## Findings

### Issues Discovered
1. [Issue description] - [Link to GitHub issue if created]

### Improvements Identified
1. [Improvement idea]

### Edge Cases
1. [Edge case description]

## Cleanup

```bash
# Commands used to clean up
docker-compose -f docker-compose.mill-test.yml down -v
cd ../..
rm -rf .dogfood/project-a/repo
```

## Next Steps

- [ ] Test additional refactoring operations
- [ ] Test on larger files
- [ ] Test with different code patterns
