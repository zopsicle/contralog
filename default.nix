let
    pkgs = import ./nix/pkgs.nix {};
in
    [
        pkgs.cargo
        pkgs.gcc
    ]
