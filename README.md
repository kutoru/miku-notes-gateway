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
- Translation layer between REST requests and gRPC for the app

More info will be added as more progress will get made on the project
