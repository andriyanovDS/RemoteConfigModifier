<div align="center">

# RemoteConfigModifier
RCM is a command-line tool to modify Firebase remote config.
</div>

### Documentation quick links
* [Installation](#installation)
* [How to use](#usage)
* [Development](#development)

<a id="installation">
<h2>Installation</h2>
</a>
RCM now is only available for MacOS.

You can build and install it from source (requires the latest stable [Rust] compiler.)
```console
cargo install --git https://github.com/andriyanovDS/RemoteConfigModifier.git rcm
```

[rust]: https://www.rust-lang.org

<a id="usage">
<h2>How to use</h2>
</a>

### Add parameter:
To add parameter run `add` subcommand with optional `-n | --name` and `-d | --description` arguments.
```shell
$ rcm add -n=new_parameter_name -d="Very useful feature flag"
```
If you have multiple projects added to `rcm`, you can specify project you want to update using `-p | --project` argument. 
```shell
$ rcm add -n=new_parameter_name -p=my_project
```
If `parameter` argument is not passed `rcm` will make attempt to add parameter to all projects.
Program will ask you value type, default and optional conditional values. 
### Update parameter
To update the parameter run `update` subcommand with required `-n | --name` argument.
```shell
$ rcm update -n=existing_parameter_name
```
You can also specify project if you want update only parameter only in specific one.
Pass `-m | --main` argument with project name you want to update first.
Parameters for other projects will inherit description and conditions. 
```shell
$ rcm update -n=existing_parameter_name -m=project_will_run_first
```

### Delete parameter
To delete parameter in all projects run `delete` subcommand with required `-n | --name` argument.
```shell
$ rcm delete -n=existing_parameter_name
```

### Move parameter to group
Remote config allow you to [group parameters](https://firebase.google.com/docs/remote-config/parameters#parameter_groups) together.
To move parameter to the group, even from another one, run `move-to` command with required `-n | --name`
and optional `-g | --group` arguments.
```shell
$ rcm move-to -n=existing_parameter_name -g="Group name"
```

### Move parameter out of the group
To move parameter out of the group run `move-out` subcommand with `-n | --name` argument.
```shell
$ rcm move-out -n=existing_parameter_name
```

### View configuration
To view configuration run `show` command with optional `-p | --project` argument.
By default, it will display all projects in separate tables. 
```shell
$ rcm show -p=my_project
```

<a id="development">
<h2>Development</h2>
</a>

`rcm` is written in [Rust](https://www.rust-lang.org/).
The recommended way to install Rust for development is from the [official download page](https://www.rust-lang.org/tools/install), using rustup.

Once Rust is installed, you can compile `rcm` with Cargo:

    cargo build
    cargo build --release
