# Local Development

1. Create dummy certs

Right now the server spins up as a TLS server so we need some dummy certs. The
certs directory is configured through the `WEBHOOK_CERTS_DIR` environment
variable so you can put them wherever you want but this case just puts them in
the `./certs/` directory (which is gitignore'd).

**IMPORTANT**: You can leave all fields blank except for the `Common Name`! Set
that to `localhost`.

```bash
$ mkdir -p certs
$ openssl req -x509 -nodes -days 365 -newkey rsa:2048 -keyout ./certs/tls.key -out ./certs/tls.crt

Generating a 2048 bit RSA private key
........+++++
............................+++++
writing new private key to 'tls.key'
-----
You are about to be asked to enter information that will be incorporated
into your certificate request.
What you are about to enter is what is called a Distinguished Name or a DN.
There are quite a few fields but you can leave some blank
For some fields there will be a default value,
If you enter '.', the field will be left blank.
-----
Country Name (2 letter code) []:
State or Province Name (full name) []:
Locality Name (eg, city) []:
Organization Name (eg, company) []:
Organizational Unit Name (eg, section) []:
Common Name (eg, fully qualified host name) []:localhost # MAKE SURE THIS IS localhost!!
Email Address []:
```

2. Run the application

Make sure to set the `WEBHOOK_CERTS_DIR` environment variable, the default
directory is `/certs` so you'll get an error if you try to run it without
the variable

```bash
WEBHOOK_CERTS_DIR=$PWD/certs/ cargo run
```

3. Interact with the server

You can now use an http client (like curl or postman) to interact with
the server.

```bash
curl -vk https://0.0.0.0:8443/livez
```

To post an admission request

```bash
curl -k -XPOST -H'content-type: application/json' -d @data/admission.json https://0.0.0.0:8443/mutate
```
