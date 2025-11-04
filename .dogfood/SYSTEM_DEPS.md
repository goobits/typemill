# System Dependencies Tracker

Track system packages installed for dogfooding projects to document container contamination.

## Format

```
## project-name (YYYY-MM-DD)
- package-name - reason
- another-package - reason
```

## Installed Dependencies

<!-- Add entries below as projects are tested -->

## Notes

- This container is already isolated from host via Docker
- System package sharing across projects is acceptable
- Document here for transparency and reproducibility
- If contamination becomes problematic, consider dedicated VMs per project
