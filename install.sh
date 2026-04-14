#!/usr/bin/env bash
# Install ai-workflow-skills from GitHub Releases (Linux x86_64 tarball) or build on macOS.
# Set GITHUB_REPOSITORY=owner/repo for your fork (default placeholder below).
set -euo pipefail

PKG_NAME="ai-workflow-skills"
PLATFORM="linux-x86_64"
GITHUB_REPOSITORY="${GITHUB_REPOSITORY:-watasabi/ai-workflow-skills}"
REPO_GIT="${REPO_GIT:-https://github.com/${GITHUB_REPOSITORY}.git}"
CATALOG_GIT="${CATALOG_GIT:-}"

VERSION="latest"
PREFIX="${HOME}/.local"
TOKEN="${GITHUB_TOKEN:-${GH_TOKEN:-}}"

usage() {
  cat <<'EOF'
Usage:
  curl -fsSL https://raw.githubusercontent.com/watasabi/ai-workflow-skills/main/install.sh | bash -s -- [options]

  -v TAG     Version tag (e.g. v0.3.0). Default: latest GitHub release.
  -p DIR     Install prefix (binaries in DIR/bin). Default: ~/.local
  -t TOKEN   Optional. GITHUB_TOKEN for private repos / API (also reads GITHUB_TOKEN or GH_TOKEN env).
  -c URL     Git URL for AI_WORKFLOW_SKILLS_CATALOG (written to shell rc if set).
  -h         This help.

  Linux x86_64: download release tarball from GitHub.
  macOS: clone tagged repo and run cargo build --release (Rust required).

  Set GITHUB_REPOSITORY=owner/repo to point at your GitHub fork before piping the script.
EOF
  exit 0
}

while getopts "v:p:t:c:h" opt; do
  case "$opt" in
    v) VERSION="$OPTARG" ;;
    p) PREFIX="$OPTARG" ;;
    t) TOKEN="$OPTARG" ;;
    c) CATALOG_GIT="$OPTARG" ;;
    h) usage ;;
    *) usage ;;
  esac
done

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

BIN_DIR="${PREFIX}/bin"
API_ROOT="https://api.github.com/repos/${GITHUB_REPOSITORY}"

api_curl() {
  local url="$1"
  if [[ -n "${TOKEN}" ]]; then
    curl -fsSL --header "Authorization: Bearer ${TOKEN}" --header "Accept: application/vnd.github+json" "${url}"
  else
    curl -fsSL --header "Accept: application/vnd.github+json" "${url}"
  fi
}

resolve_version() {
  if [[ "${VERSION}" != "latest" ]]; then
    echo "${VERSION}"
    return
  fi
  local json tag
  json=$(api_curl "${API_ROOT}/releases/latest") || true
  tag=$(printf '%s' "${json}" | python3 -c "
import json, sys
try:
    d = json.load(sys.stdin)
    print(d.get('tag_name', '') or '')
except Exception:
    print('')
" 2>/dev/null || true)
  if [[ -z "${tag}" ]]; then
    echo -e "${RED}Could not resolve latest release. Set -v TAG or GITHUB_REPOSITORY, or use a public repo.${NC}" >&2
    exit 1
  fi
  echo "${tag}"
}

download_binary() {
  local ver="$1"
  local url="https://github.com/${GITHUB_REPOSITORY}/releases/download/${ver}/${PKG_NAME}-${PLATFORM}-${ver}.tar.gz"
  local tmp
  tmp="$(mktemp -d)"
  echo -e "${BLUE}Downloading ${PKG_NAME} ${ver}…${NC}"
  if [[ -n "${TOKEN}" ]]; then
    curl -fsSL --header "Authorization: Bearer ${TOKEN}" "${url}" -o "${tmp}/dist.tar.gz"
  else
    curl -fsSL "${url}" -o "${tmp}/dist.tar.gz"
  fi
  tar -xzf "${tmp}/dist.tar.gz" -C "${tmp}"
  mkdir -p "${BIN_DIR}"
  local artifact="${tmp}/${PKG_NAME}"
  [[ -f "${artifact}" ]] || artifact="${tmp}/ai-workflow-skills"
  [[ -f "${artifact}" ]] || {
    echo -e "${RED}Binary not found in tarball (expected ${PKG_NAME} or ai-workflow-skills).${NC}" >&2
    rm -rf "${tmp}"
    exit 1
  }
  install -m 0755 "${artifact}" "${BIN_DIR}/${PKG_NAME}"
  rm -rf "${tmp}"
  echo -e "${GREEN}Installed ${BIN_DIR}/${PKG_NAME}${NC}"
}

build_and_install_darwin() {
  local ver="$1"
  local work clone_url
  work="$(mktemp -d)"
  clone_url="${REPO_GIT}"
  if [[ -n "${TOKEN}" ]] && [[ "${clone_url}" =~ ^https://github.com/ ]]; then
    clone_url="https://${TOKEN}@github.com/${clone_url#https://github.com/}"
  fi
  echo -e "${BLUE}Cloning ${PKG_NAME} (${ver}) for macOS build…${NC}"
  if ! git clone --depth 1 --branch "${ver}" "${clone_url}" "${work}"; then
    echo -e "${RED}Clone failed (missing tag or private repo: set -t or GITHUB_TOKEN).${NC}" >&2
    rm -rf "${work}"
    exit 1
  fi
  echo -e "${BLUE}Running cargo build --release --locked…${NC}"
  if ! (cd "${work}" && cargo build --release --locked); then
    echo -e "${RED}cargo build failed. Install Rust: https://rustup.rs/${NC}" >&2
    rm -rf "${work}"
    exit 1
  fi
  mkdir -p "${BIN_DIR}"
  install -m 0755 "${work}/target/release/${PKG_NAME}" "${BIN_DIR}/${PKG_NAME}"
  rm -rf "${work}"
  echo -e "${GREEN}Installed ${BIN_DIR}/${PKG_NAME}${NC}"
}

ensure_env_snippet() {
  [[ -n "${CATALOG_GIT}" ]] || return 0
  local line="export AI_WORKFLOW_SKILLS_CATALOG=\"${CATALOG_GIT}\""
  local rc=""
  case "${SHELL##*/}" in
    zsh) rc="${ZDOTDIR:-${HOME}}/.zshrc" ;;
    *) rc="${HOME}/.bashrc" ;;
  esac
  touch "${rc}"
  local tmp
  tmp="$(mktemp)"
  if [[ -s "${rc}" ]]; then
    grep -Ev '^# ai-workflow-skills — catalog|^export AI_WORKFLOW_SKILLS_CATALOG=' "${rc}" > "${tmp}" || true
    mv "${tmp}" "${rc}"
  else
    rm -f "${tmp}"
  fi
  {
    echo ""
    echo "# ai-workflow-skills — AI_WORKFLOW_SKILLS_CATALOG (install.sh); ai-workflow-skills syncs to ~/.cache/.../catalog-git/"
    echo "${line}"
  } >> "${rc}"
  echo -e "${GREEN}AI_WORKFLOW_SKILLS_CATALOG set in ${rc}:${NC}"
  echo "  ${line}"
  if ! echo "${PATH}" | grep -qF "${BIN_DIR}"; then
    if ! grep -qF "export PATH=\"${BIN_DIR}" "${rc}" 2>/dev/null; then
      echo "" >> "${rc}"
      echo "# ai-workflow-skills — PATH (install.sh)" >> "${rc}"
      echo "export PATH=\"${BIN_DIR}:\${PATH}\"" >> "${rc}"
      echo -e "${GREEN}PATH updated in ${rc}${NC}"
    fi
  fi
  echo -e "${BLUE}Open a new shell or: source ${rc}${NC}"
}

OS="$(uname -s)"
ARCH="$(uname -m)"
INSTALL_KIND=""

if [[ "${OS}" == "Linux" ]] && [[ "${ARCH}" == "x86_64" ]]; then
  INSTALL_KIND="linux_binary"
elif [[ "${OS}" == "Darwin" ]]; then
  INSTALL_KIND="darwin_build"
else
  echo -e "${RED}Supported: Linux x86_64 (release tarball) or macOS (local build). System: ${OS} ${ARCH}${NC}" >&2
  exit 1
fi

command -v curl >/dev/null || {
  echo -e "${RED}curl is required.${NC}" >&2
  exit 1
}
command -v python3 >/dev/null || {
  echo -e "${RED}python3 is required to resolve latest release.${NC}" >&2
  exit 1
}
if [[ "${INSTALL_KIND}" == "darwin_build" ]]; then
  command -v git >/dev/null || {
    echo -e "${RED}git is required on macOS.${NC}" >&2
    exit 1
  }
  command -v cargo >/dev/null || {
    echo -e "${RED}cargo is required on macOS. Install Rust: https://rustup.rs/${NC}" >&2
    exit 1
  }
fi

VER_RESOLVED="$(resolve_version)"
echo -e "${BLUE}Version: ${VER_RESOLVED}${NC}"

case "${INSTALL_KIND}" in
  linux_binary) download_binary "${VER_RESOLVED}" ;;
  darwin_build) build_and_install_darwin "${VER_RESOLVED}" ;;
esac

ensure_env_snippet

echo ""
echo -e "${GREEN}Done.${NC}"
echo -e "${BLUE}Interactive UI:${NC} ${BIN_DIR}/${PKG_NAME}"
