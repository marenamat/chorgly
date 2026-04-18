# Questions / Action items for guardian

## Docker Hub secrets needed (issue #3)

The Docker CI workflow (`docker.yml`) requires two repository secrets to be
configured in GitHub:

- `DOCKERHUB_USERNAME` — Docker Hub account name
- `DOCKERHUB_TOKEN` — Docker Hub access token (read/write)

Without these the "Log in to Docker Hub" step will always fail and the image
will never be pushed. Please add them under
**Settings → Secrets and variables → Actions** in the `marenamat/chorgly`
repository.
