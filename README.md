<h1 align="center">Dekinai - Minimalistic Uploading API</h1>

<p align="center">
  <i>Dekinai is a private project used by myself although available to anyone who's interested.</i>
</p>

## Features

-   Upload files.
    -   Uploaded files will be renamed to a random alphanumeric name, but will preserve their original file extension (if there was any).
-   Delete uploaded files through a deletion URL (received after uploading your file).
-   Blacklist file extensions.

## Notes

-   Dekinai is intended to be reverse proxied by a web server like `nginx` or `apache`.
-   Dekinai only handles uploads.

How the uploaded files are served is up to your web server. I am not responsible for any security issues that may occur if you do not blacklist or prevent the execution of e.g. uploaded `.php` files on your server.

## Usage

Just run `dekinai.exe your_uploads_folder`. Dekinai will start the server and save all uploaded files in that folder.

For more information try `--help`:

```
$ dekinai.exe --help

Dekinai 1.0.0

Johann Rekowski <johann.rekowski@gmail.com>

Dekinai is a simple and minimalistic file uploading API.
It provides the bare minimum feature set for file hosting services, while also being fast and portable.

USAGE:
    dekinai.exe [OPTIONS] <OUTPUT_DIR>

ARGS:
    <OUTPUT_DIR>    Sets the directory for uploaded files

OPTIONS:
    -p, --port <NUMBER>                    Sets the port number to listen on [default: 54298]
    -u, --unix <FILE>                      Sets the unix socket to listen on (Unix only)
    -d, --database <DATABASE_DIR>          Sets the directory for the SQLite database [default: ./]
        --password <TEXT>                  Sets a password for API requests [env: DEKINAI_PASSWORD=]
    -b, --blacklist <FILE_EXTENSION>...    Sets a list of disallowed file extensions
                                           Usage: --blacklist asp html php
        --disable-port                     Disables port listening (Unix only)
    -h, --help                             Print help information
    -V, --version                          Print version information
```

## API

-   **[POST] /**
    -   This route expects multipart data.
        -   The multipart data must consist of just one field that is carrying the uploaded file.
        -   The name of the field is not important, the server will try to accept the first available multipart field in the request and will ignore any additional fields.
        -   The file field needs to have content disposition, describing the filename. This is usually automatically set by whatever software you are using to upload the file. However, should it be missing, the request will fail with status code 400.
    -   This route will respond with a JSON string array.
        -   The JSON array has always a length of `2`.
        -   Index `0` contains the URL for your uploaded file.
        -   Index `1` contains the deletion URL for your uploaded file.
    -   Optional: When the `--password` option is being used, this route will check requests for an `X-API-Key` header.
        -   If the `X-API-Key` header is missing or does not contain the correct password, the request will fail with status code 401.
        -   Example header: `X-API-Key: MyPassword`

-   **[GET] /`filename`/`deletion_password`**
    -   This route will delete the requested file and respond with _"File has been deleted."_ whenever `filename` and `deletion_password` match.

## Proxy Headers

You might have already noticed that there is no configuration option for the response base URL. This is because Dekinai expects certain headers from your web server to build the correct response base URL.

If no proxy headers are provided, the default response base URL will be: http://localhost:54298/

To change it into your desired base URL provide these headers to Dekinai when reverse proxying:

-   `X-Forwarded-Proto`
    -   Example: `X-Forwarded-Proto: https`
-   `Host`
    -   Example: `Host: www.dekinai.moe`
-   Optional: `X-Forwarded-Path`
    -   Example: `X-Forwarded-Path: /downloads`

Assuming you provided all of these headers, the response base URL would look like this: https://www.dekinai.moe/downloads/

If you now upload a file, the response will look like this:

```json
[
    "https://www.dekinai.moe/downloads/uploadedFile.png",
    "https://www.dekinai.moe/downloads/uploadedFile.png/VeryRandomPassword"
]
```

## Example Nginx Config

```nginx
upstream dekinai {
    # Use this if you are using the unix socket option (I do):
    server unix:/run/dekinai/dekinai.sock;

    # Use this if you are using the reqular port (change the port number if needed):
    # server localhost:54298;
}

server {
    listen 80;
    server_name dekinai.moe;
    root /the/path/to/where/dekinai/saves/files;

    # Set the proxy headers:
    proxy_set_header X-Forwarded-Proto $scheme;
    proxy_set_header Host $host;
    proxy_http_version 1.1;

    location / {
        # Try to find a file at current path in root folder and if it doesn't work load /index.html at
        # whatever path you are at.
        try_files $uri $uri/ /index.html;

        # Optional: Any file found in root folder that ends with htm or html will be forced to download.
        # Only necessary if you don't blacklist these file types.
        if ($request_filename ~* ".+\.(?:htm|html)$") {
            add_header Content-Disposition attachment;
        }
    }

    # Optional: Use this empty location block if you force downloads of html files, otherwise you won't be
    # able to view the index file.
    location = /index.html {}

    # Upload location of Dekinai. Note the / (slash) sign at the end of proxy_pass! It will make requests
    # from the root path if the slash at the end is set.
    location = /upload {
        proxy_pass http://dekinai/;
        client_max_body_size 1G;
    }

    # Regex location for deletion URLs. Note no / (slash) sign at the end of proxy_pass, because I want
    # Dekinai to receive the current path here.
    location ~* ^/[^/]+/[^/]+/?$ {
        proxy_pass http://dekinai;
    }

    # Logging
    error_log /var/log/nginx/dekinai-error.log warn;
}
```

## Example Systemd Config

```systemd
[Unit]
Description=Dekinai - Minimalistic Uploading API
Wants=network-online.target
After=network-online.target

[Service]
User=dekinai
Group=nginx
UMask=0002
WorkingDirectory=/opt/apps
RuntimeDirectory=dekinai
RuntimeDirectoryMode=2775
ExecStart=/opt/apps/dekinai -u /run/dekinai/dekinai.sock -d /opt/apps/dekinai-database --disable-port /the/path/to/where/dekinai/saves/files
Restart=always
RestartSec=10
SyslogIdentifier=dekinai
# Environment=DEKINAI_PASSWORD="You can add this to define a password through environment variables instead of using --password"

[Install]
WantedBy=multi-user.target
```
