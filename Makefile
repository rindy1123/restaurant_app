API_CONTAINER_NAME := $(shell docker ps | grep api | cut -d ' ' -f1)

test:
	docker exec $(API_CONTAINER_NAME) cargo test
