{ pkgs, ... }:
{
  packages = [
    pkgs.openssl
    pkgs.pkg-config
  ];

  languages = {
    rust.enable = true;
    javascript.enable = true;
  };

  env = {
    OPENSSL_DIR = "${pkgs.openssl.dev}";
    OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
    OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
  };
}
