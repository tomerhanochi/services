# Build a container image for a workspace binary.
build bin:
    docker build -f {{ source_directory() }}/Containerfile --build-arg BIN={{ bin }} -t {{ bin }} {{ justfile_directory() }}
