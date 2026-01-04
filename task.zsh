#!/bin/zsh
# set -x

local_env=(
  TMPDIR=/tmp
)

# run from script directory
cd $(dirname $0:P) 

_cargo() { local args="$@"; sh -xc "${local_env[*]} cargo $args"; }

clippy() {
  _cargo run -F codegen
  _cargo clippy --all-targets -F codegen --workspace
}

test() {
  _cargo run -F codegen
  _cargo test -F codegen --workspace
}

release-prep() {
  release-plz update
  new_version=$(cargo pkgid | rev | cut -d / -f 1 | rev | cut -d '#' -f 2)
  git commit -a -m "v${new_version}"
  git tag -a v${new_version} -m "v${new_version}" HEAD
}

BIN=$(basename $0)
TASKS=("${(@f)$(print -l ${(ok)functions[(I)[^_]*]})}")
_usage() { local sep=' | '; >&2 echo "usage: $BIN ( $(print -R ${(pj|$sep|)TASKS}) )" }

if [[ $# -lt 1 ]]; then; _usage; exit 1; fi
if ! (($TASKS[(Ie)$1])); then; >&2 echo "unknown task: $1"; _usage; exit 1; fi
TASK=$1; shift; $TASK $@
