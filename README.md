# frontend-host
`frontend-host` is a simple web server that can be used to test the frontend of an application without needing any working backend APIs.

Appropriate API responses can be simulated by placing JSON files in an `api` directory, which maps directly to URLs which may be called by the frontend application.

## Usage

1) Create `api` and `static` directories.
2) Place JSON files in the `api` directory. JSON files map directly to a URL, so the URL `/api/test` will map to the `api/test.json` file. Subdirectories can be added as well.
3) Make sure all your frontend assets are available in the `static` directory.
4) Run the application. By default, accessing `localhost` will load an `index.html` from the `static` directory, otherwise it will print a banner page.

## License
`frontend-host` is dual licensed under MIT & Apache 2.0