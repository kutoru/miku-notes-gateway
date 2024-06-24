# ?

This is one out of four parts of [Miku notes]().

Application parts:
- [Auth service](https://github.com/kuromii5/sso-auth)
- [Data service](https://github.com/kutoru/miku-notes-data)
- [Gateway service](https://github.com/kutoru/miku-notes-gateway) (this repo)
- [Frontend](https://github.com/kinokorain/Miku-notes-frontend)

This repo is something like an API gateway that acts as a:
- Proxy for the Auth service and the Data service
- Single point of entry for the Frontend
- Auth validation layer for the app
- Translation layer between REST requests and gRPC requests for the app

# How to run

First, make sure that you:
- have cloned the submodule in the `./proto` directory
- have the [protoc](https://grpc.io/docs/protoc-installation) binary on your path
- have created and filled out your [.env configuration](#env)
- have both **Auth service** and **Data service** launched and running on URLs according to your .env configuration
- optionally, have either the **Frontend** running, or make requests manually via things like curl or Postman

After that you can do the usual `cargo run` in the root directory

# .env

The .env file should be located in the root directory and have the following contents:
```
SERVICE_ADDR=127.0.0.1:33033
FRONTEND_URL=http://localhost:5173
AUTH_URL=http://127.0.0.1:44044
DATA_URL=http://127.0.0.1:55055
MAX_REQUEST_BODY_SIZE_IN_MB=4096
MAX_FILE_CHUNK_SIZE_IN_MB=10
TOKEN_EXP=30
```
Where:
- `SERVICE_ADDR` is the address that this service will run on
- `FRONTEND_URL` is the url that the **Frontend** is running on. Required for CORS stuff
- `AUTH_URL` is the url that the **Auth service** is running on
- `DATA_URL` is the url that the **Data service** is running on
- `MAX_REQUEST_BODY_SIZE_IN_MB` is an unsigned int that will become the maximum allowed size (in megabytes) for received multipart request bodies. Files received from multipart bodies never get loaded into memory, so big numbers (up to like 50GB) should in theory be fine as a value for this field, although big files like that will take a long time to get uploaded
- `MAX_FILE_CHUNK_SIZE_IN_MB` is an unsigned int that will become the maximum allowed size (in megabytes) for sent file chunks in gRPC messages. **IMPORTANT**: this value should be identical to or smaller than the **Data service**'s .env value with the identical key name, or else stuff will go wrong
- `TOKEN_EXP` is an int that will become the token cookie expiry time (in seconds). Note that this value is currently only used for testing purposes, and as such will get removed later
