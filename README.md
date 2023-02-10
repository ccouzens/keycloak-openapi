# Keycloak-Admin-OpenAPI-Definition

[OpenAPI definitions](https://github.com/OAI/OpenAPI-Specification) for
[Keycloak](https://www.keycloak.org/)'s Admin API.

These can be used to generate libraries for interacting with Keycloak from any
mainstream programming langauge.

OpenAPI definitions are sometimes known by their previous name of Swagger
specifications.

[Keycloak 20.0.3 Admin API](./keycloak/20.0.3.json)

[Keycloak 13.0 Admin API with patches](./keycloak/13.0-patched.json)

[Keycloak 12.0 Admin API with patches](./keycloak/12.0-patched.json)

[Other versions](./keycloak/)

[Red Hat Single Sign-On](https://access.redhat.com/products/red-hat-single-sign-on)
Admin API definitions are not distributed in this repository, but can be
generated.

```bash
make keycloak/sso-6.json keycloak/sso-7.3.json keycloak/sso-7.4.json
```

## Alternatives

DAHAG Rechtsservices AG
[provide OpenAPI definitions](https://github.com/dahag-ag/keycloak-openapi) for
recent versions of Keycloak which are more complete than what's here.

## Account API

Keycloak also has an undocumented [Account API](https://github.com/keycloak/keycloak/blob/main/services/src/main/java/org/keycloak/services/resources/account/AccountRestService.java) that is used by the internal account console to display and update user information. This is be useful for building a custom account console, or building account info/editing into an application. [@xgp](https://github.com/xgp) has created a [Gist of OpenAPI definitions of the Account API](https://gist.github.com/xgp/2d77cbebc6164160faae6aa77d127a57).

## General Usage

Typically to use Keycloak's admin Rest API, you first get a token from a realm.
This is done using the OAuth2 protocol.

```bash
bearer_token=$(curl -X POST 'http://localhost:8080/auth/realms/master/protocol/openid-connect/token' \
     --data-urlencode 'username=admin-user' \
     --data-urlencode 'password=admin-password' \
     --data-urlencode 'grant_type=password' \
     --data-urlencode 'client_id=admin-cli' \
      | jq -r '.access_token')
```

The realm used for the token is not necessarily the realm used in the API.

The endpoints in this specification can then be used with a base url of the form
`http://localhost:8080/auth/admin/realms` and the above bearer token.

Please see the [example app](example_app/src/index.ts).

## Help wanted

The definitions are computer generated, but
[could do with a human's input](https://github.com/ccouzens/keycloak-openapi/issues/10).

[Please provide additional example apps](https://github.com/ccouzens/keycloak-openapi/issues/14).

## Adding new versions

To add a new version of Keycloak perform these steps:

1. Modify the [Makefile](./Makefile) with the
   [new version](https://github.com/ccouzens/keycloak-openapi/commit/6f717533b13cd07bae3c4aa1f55907fb40338540#diff-76ed074a9305c04054cdebb9e9aad2d818052b07091de1f20cad0bbac34ffb52).
2. Verify you have installed the [requirements](#requirements).
3. Run the command `make`. It will automatically:
   1. download the documentation in HTML format
   2. compile the [transformer](#keycloak-openapi-transformer)
   3. create the JSON definition
   4. create the YML definition
4. Commit your changes and open a pull request.

## Notable Changes

2021 Early - Renamed master branch to main. Anyone fetching the schema directly
from the branch should update their reference.

2020 May - Added tags to operations. Users of OpenAPI Generator generated
clients will need to swap from using the `DefaultApi` class to multiple classes
such as `UsersApi` or `ClientsApi`.

## Example App

[This example](./example_app/) uses
[openapi-generator](https://github.com/OpenAPITools/openapi-generator) to build
an API-client. The app in the example is very simple and only fetches a list of
Keycloak-clients.

```bash
# Create a keycloak instance to run against
docker container run --rm -e KEYCLOAK_USER=admin-user -e KEYCLOAK_PASSWORD=admin-password -p 8080:8080 docker.io/jboss/keycloak:13.0.0
# Open http://localhost:8080/ and wait for keycloak to start up

# In a second terminal run
cd example_app
npm install
npm run generate-client
npm run compile
node dist/index.js
# prints out
# The default clients:
#   account
#   account-console
#   admin-cli
#   broker
#   master-realm
#   security-admin-console
```

In general, clients can be generated by running a command similar to:

```bash
openapi-generator generate -i 'https://github.com/ccouzens/keycloak-openapi/raw/main/keycloak/20.0.3.json' -g 'typescript-axios' -o 'src/keycloak-client'
```

or using docker, if you can't install `openapi-generator`:

```bash
docker run --rm --user $(id -u):$(id -g) -v $PWD:/local openapitools/openapi-generator-cli generate -i 'https://github.com/ccouzens/keycloak-openapi/raw/main/keycloak/20.0.3.json' -g typescript-axios -o /local/src/keycloak-client
```
## Keycloak OpenAPI Transformer

All the OpenAPI definitions are generated from the published HTML documentation.
This tool transforms the HTML documentation into OpenAPI definitions.

HTML of the documentation is inputted through `stdin` and the JSON of the
definition is outputted through `stdout`.

## Requirements

Building Red Hat Single Sign-On definitions or rebuilding Keycloak's definitions
requires:

- `make`
- [`Rust`](https://www.rust-lang.org/tools/install)
- [`yq`](https://github.com/mikefarah/yq/)

Building the transformer requires
[`Rust`](https://www.rust-lang.org/tools/install).

Building the example app requires `Node` and `Java`.

Running the keycloak instance that pairs with the example app requires `Docker`.

## Licensing

The OpenAPI definitions are [Apache 2.0 licensed](./keycloak/LICENSE.txt).

The transformer is [MIT licensed](keycloak-openapi-transformer/LICENSE).

The example app has a [CC0 waiver](example_app/WAIVER) (has had its copyright
waived).
