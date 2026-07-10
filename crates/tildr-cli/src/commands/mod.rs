use clap::Subcommand;

mod add;
mod apply;
mod cat;
mod completions;
mod convert;
mod del;
mod doctor;
mod edit;
mod exclude;
mod git;
mod import;
mod info;
mod init;
mod list;
mod mv;
mod repo;
mod restore;
mod secret;
mod status;
mod sync;
mod unlink;

#[derive(Subcommand, Debug, Clone)]
pub enum CliCommands {
  Init(init::Command),
  Add(add::Command),
  Restore(restore::Command),
  Unlink(unlink::Command),
  List(list::Command),
  Apply(apply::Command),
  Repo(repo::Command),
  Cat(cat::Command),
  Completions(completions::Command),
  Status(status::Command),
  Doctor(doctor::Command),
  Edit(edit::Command),
  Secret(secret::Command),
  Info(info::Command),
  Git(git::Command),
  Del(del::Command),
  Sync(sync::Command),
  Import(import::Command),
  Mv(mv::Command),
  Exclude(exclude::Command),
}
