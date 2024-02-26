Nix Hash Collection
===============================

Software to centralize build-output hashes from several builders.

Composed of 2 parts:
1) A post-build-hook
2) A server to aggregate the results

### Howto

#### Server

- Run the server with `uvicorn web:app --reload`
- Create a user token by running `./create_user "username"`

#### Client

```nix
  services.hash-collection = {
    enable = true;
    collection-url = "server url";
    tokenFile = "/token/path";
  };
```

##### Signing

Set up your keys with:

* `nix key generate-secret --key-name hash-collection > secret.key`
* `nix key convert-secret-to-public < ./secret-key > ./public-key`

### TODO:
- [ ] method to ingest hydra's results
- [ ] catch all sort of errors
