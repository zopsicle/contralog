let
    tarball = fetchTarball {
        url = "https://github.com/NixOS/nixpkgs/archive/f5cc5ce8d60fe69c968582434fbfbf8f350555cb.tar.gz";
        sha256 = "025773zp9hvizwf4frimm7mnr6cydmckw7kayqmik6scisq0mfk5";
    };
    config = {};
in
    {}: import tarball {inherit config;}
