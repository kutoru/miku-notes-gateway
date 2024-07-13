# ?

This is one out of four parts of [Miku Notes](https://github.com/kutoru/miku-notes). This part is something like an API gateway that acts as a:
- Proxy for the **Auth service** and the **Data service**
- Single point of entry for the **Frontend**
- Auth validation layer for the app
- Translation layer between REST requests and gRPC requests for the app

Once you have successfully started the service, the documentation for each API route will be available at `http://localhost:{SERVICE_PORT}/swagger-ui` or `http://localhost:{SERVICE_PORT}/scalar`

# How to run this service

**It is highly recommended** to run this service along with other parts simultaneously by using docker compose. The instructions for that can be found in the [main repository](https://github.com/kutoru/miku-notes).

With having that said, you could still run the service manually by following the instructions below.

First, make sure that you:
- have cloned the submodule in the `./proto` directory
- have the [protoc](https://grpc.io/docs/protoc-installation) binary on your path
- have created and filled out your [.env configuration](#env)
- have both **Auth service** and **Data service** set up and running on URLs according to your .env configuration
- optionally, have either the **Frontend** running, or make requests manually via things like curl or Postman

After that you can do the usual `cargo run` in the root directory

# .env

The .env file should be located in the root directory and have the following contents:
```
LOG_LEVEL=info
SERVICE_PORT=3030
FRONTEND_URL=http://localhost:5173
MAX_REQUEST_BODY_SIZE=8192
MAX_FILE_CHUNK_SIZE=8

AUTH_URL=http://127.0.0.1:4040
DATA_URL=http://127.0.0.1:5050
AUTH_TOKEN=7osu2game7
DATA_TOKEN=39sankyu39

ACCESS_TOKEN_TTL=60
REFRESH_TOKEN_TTL=300
ACCESS_TOKEN_KEY=at
REFRESH_TOKEN_KEY=rt
```
Where:
- `LOG_LEVEL` is the log level for the service. Can be either `debug`, `info` or `error`
- `SERVICE_PORT` is the port that this service will run on
- `FRONTEND_URL` is the url that the **Frontend** is running on. Required for CORS stuff
- `MAX_REQUEST_BODY_SIZE` is an unsigned int that will become the maximum allowed size (in megabytes) for received multipart request bodies. Files received from multipart bodies never get fully loaded into memory, so big numbers (up to like 50GB) should in theory be fine as a value for this field, although big files like that will take a long time to get uploaded
- `MAX_FILE_CHUNK_SIZE` is an unsigned int that will become the maximum allowed size (in megabytes) for sent file chunks in gRPC messages. **Note** that this value should be identical to the **Data service**'s .env value with the same key name, or else the service won't be able to send/decode gRPC messages

- `AUTH_URL` is the url that the **Auth service** is running on
- `DATA_URL` is the url that the **Data service** is running on
- `AUTH_TOKEN` is a string that will be passed as a bearer token along with each request to the **Auth service**
- `DATA_TOKEN` is a string that will be passed as a bearer token along with each request to the **Data service**

- `ACCESS_TOKEN_TTL` is an int that will become the access cookie's expiry time (in seconds). Should probably have the same value with the `access_ttl` key in the **Auth service**
- `REFRESH_TOKEN_TTL` is an int that will become the refresh cookie's expiry time (in seconds). Should probably have the same value with the `refresh_ttl` key in the **Auth service**
- `ACCESS_TOKEN_KEY` is a short string that will become the access cookie's key 
- `REFRESH_TOKEN_KEY` is a short string that will become the refresh cookie's key 
