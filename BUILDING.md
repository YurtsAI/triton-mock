To release, use the Makefile targets:

```bash
make release RELEASE=patch EXECUTE=y
```

Publishing a new docker image is done manually:

```bash
make docker-build docker-publish
```
