# Crate features
This crate offers to compile the underlying [`libzmq`](https://github.com/zeromq/libzmq) with different configurations 
based on your needs by enabling or disabling cargo features. In addition to that, there are several 
[`Rust-related features`](#rust-related-features) that you can enable or diable.

[`Below`](#libzmq-related-features) there is a brief overview, with steps for `Windows`, `Linux`, and `MacOs` for the 
underlying dependencies for `libzmq`. Other platforms might work as well, but are untested at this point. Let me know 
if you made experiences on unmentioned platforms, so that I can provide hints for others in similar situations.

## Rust-related features
The main Rust-related features are [`builder`](#builder), and [`futures`](#futures). [`builder`](#builder) is enabled 
by default.

### `builder`
Enables a builder API for the variouos socket types as well as the 0MQ context. Enabled by default.

### `futures`
Enables async futures for the different send and receive traits to use with an async runner like `tokio`, `async-std`,
and the `futures` executor crate.

## `libzmq`-related features
`libzmq` offers multiple configurations to include. As it was hard for me to figure out the different compilation 
options for me to finally succeed incorporating the different features in `libzmq`, I decided to include the approaches 
I ended up with here, so that others may be able to reproduce them.

With the exceptions of [`draft-api`](#draft-api) and [`vmci`](#vmci) you will have to install additional libraries on 
your system for the feature to work. By default, the compilation will succeed if the pre-requisites are not in place, 
but you will not be able to use the functionality. For convenience, that build script generates cfg-checks that you can
use to check `libzmq` capabilities, i.e.:

```no_run,ignore
#[cfg(zmq_has = "curve")]
fn only_working_when_curve_is_enabled_and_working() { ... }
```

will only compile in the function when `curve` is enabled and working.

### `draft-api`
Enables all the functionality from the `draft-api`. This includes various socket types like `Client`, `Peer`, as well 
as several socket and context options. There are no external dependendencies necessary beyond that.

### CURVE
CURVE authoritzation is provided through libsodium that can be installed through a package on all systems.

#### Linux
On Linux, you will have to install the `libsodium-dev` package through your package manager, i.e.
```bash,ignore
sudo apt-get install libgsodium-dev
```
Library locations will be gathered through pkg-config and linked staticallyy to the underlying `libzmq` library.

#### MacOS
On MacOS, `libsodium` can be installed via package managers like homebrew:
```shell,ignore
brew install libsodium
```
Library locations will be gathered through pkg-config and linked to the underlying `libzmq` library.

#### Windows
On Windows, you need to install libsodium through vcpkg.
```shell,ignore
vcpkg install --triplet "x64-windows-md" libsodium
```
Library locations will be gathered through vcpkg and linked statically to the underlying `libzmq` library.

### GSS API
GSS API authoritzation is provided through Kerberos. Linux and Windows systems have an implementation that can be 
installed. MacOS ships with the Kerberos framework, that you will have to configure.

#### Linux
On Linux, you will have to install the `libgssapi-krb5` package through your package manager, i.e.
```bash,ignore
sudo apt-get install libgssapi-krb5
```
Library locations will be gathered through pkg-config and linked staticallyy to the underlying `libzmq` library.

#### MacOS
On MacOS, the built-in Kerberos.Framework can be used, but you have to tell the build process about it by setting the 
following environment variables:
```shell,ignore
export SYSTEM_DEPS_GSSAPI_LIB_FRAMEWORK=Kerberos
export SYSTEM_DEPS_GSSAPI_NO_PKG_CONFIG=1
```
Library locations will be gathered through the provided Framework and linked staticallyy to the underlying `libzmq` 
library.

#### Windows
On Windows, you need to install krb5 through vcpkg.
```shell,ignore
vcpkg install krb5
```
Library locations will be gathered through vcpkg and linked dynamically to the underlying `libzmq` library.

### PGM
PGM is provided through the [`openpgm`](https://github.com/steve-o/openpgm) project. Library versions 5.1, 5.2, and 5.3 
should work through pkg-config out of the box. You may have to manually compile it on some platforms and let the build 
process know of its installation through environment variables or your pkg-config configuration.

#### Linux
On Linux, you will have to install the `libpgm-dev` package through your package manager, i.e.
```bash,ignore
sudo apt-get install libpgm-dev
```
Library locations will be gathered through pkg-config and linked statically to the underlying `libzmq` library.

#### MacOS
On MacOS, you will have to compile the `OpenPGM` library from source on your own with cmake. Library locations will be
gathered through pkg-config and linked statically to the underlying `libzmq` library.

#### Windows
On Windows, you have to compile the `openpgm` library from source on your own with cmake and install it onto your
machine on your own. After that, and before compiling you need to set the following environment variables to let the
build script know where to find them:
```shell,ignore
set SYSTEM_DEPS_OPENPGM_SEARCH_NATIVE=<openpgm-installation-dir>\lib
set SYSTEM_DEPS_OPENPGM_INCLUDE=<openpgm-installation-dir>\include
set SYSTEM_DEPS_OPENPGM_LIB=libpgm-v143-mt-5_2_127
set SYSTEM_DEPS_OPENPGM_NO_PKG_CONFIG=1
```
Library locations will be gathered through these variables and linked statically to the underlying `libzmq` library.

### NACK-Oriented Reliable Multicast (NORM)
The official source code for the NORM library can be downloaded from the 
[`repository of the US Naval Research Laboratory`](https://github.com/USNavalResearchLaboratory/norm). You may have to 
manually compile it on some platforms and let the build process know of its installation through environment variables 
or your pkg-config configuration..

#### Linux
On Linux, you will have to install the `libnorm-dev` package through your package manager, i.e.
```bash,ignore
sudo apt-get install libnorm-dev
```
Library locations will be gathered through pkg-config and linked statically to the underlying `libzmq` library.

#### MacOS
On MacOS, you will have to compile the `libnorm` library from source on your own with cmake. Library locations will be 
gathered through pkg-config and linked statically to the underlying `libzmq` library.

#### Windows
On Windows, you have to compile the `libnorm` library from source on your own with cmake and install it onto your 
machine on your own. After that, and before compiling you need to set the following environment variables to let the 
build script know where to find them:
```shell,ignore
set SYSTEM_DEPS_NORM_SEARCH_NATIVE=<norm-installation-dir>\lib
set SYSTEM_DEPS_NORM_INCLUDE=<norm-installation-dir>\include
set SYSTEM_DEPS_NORM_LIB=norm
set SYSTEM_DEPS_NORM_NO_PKG_CONFIG=1
```
Library locations will be gathered through these variables and linked statically to the underlying `libzmq` library.

### `vmci`
Enables the VMware socket types for connecting to a virtual machine. There are no external dependendencies necessary 
beyond that as the functionality can be compiled in from a simple header file on all platforms.
