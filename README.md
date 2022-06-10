<div align="center">

# Firebase remote config CLI (rcm)
rcm is a command-line tool to modify Firebase remote config.
</div>

### Documentation quick links
* [How to use](#usage)
* [Development](#development)

<a id="usage">
<h2>How to use</h2>
</a>

### Setup:
It's required to store configuration JSON with Firebase projects to start using CLI.
JSON format is the following:
```typescript
{
  "projects": [
    {
      "project_number": string,
      "name": string, // project name will be only used in CLI terminal ouput
      "app_ids": [string]
    }
  ]
}
```
To store config run `config store path_to_config` subcommand
```shell
rcm config store ./config.json
```

It's also possible to add single project using command line.
Run `config add` subcommand with required `-n | --name` and `--project_number`
and optional `-a | --app_ids` arguments
```shell
rcm config add -n=project_name --project_number=project_number -a first_app_id -a second_app_id
```
To remove project run `config rm proejct_name` subcommand.
To view stored projects run `config show` subcommand with optional `-n | --name` argument.
```shell
rcm config show -n=project_name
```

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

Put `client_secret.json` with Google OAuth 2.0 secret to the root directory of the project.
Once Rust is installed and secret file is placed, you can compile `rcm` with Cargo:

    cargo build
    cargo build --release
