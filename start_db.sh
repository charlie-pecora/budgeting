#! /bin/bash

podman run --name db -e POSTGRES_USER=test -e POSTGRES_DATABASE=test -e POSTGRES_PASSWORD=test \
	-d -p 5432:5432 \
	postgres:16

