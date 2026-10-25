# Lint all Helm charts in the repository.
lint:
    find . -type d -mindepth 1 -maxdepth 1 -exec helm lint {} \;
