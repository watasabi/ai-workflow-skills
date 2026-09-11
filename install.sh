#!/usr/bin/env bash
# Install ai-workflow-skills from GitHub Releases (prebuilt tarball for Linux/macOS, x86_64/arm64).
# Set GITHUB_REPOSITORY=owner/repo for your fork (default placeholder below).
set -euo pipefail

PKG_NAME="ai-workflow-skills"
GITHUB_REPOSITORY="${GITHUB_REPOSITORY:-watasabi/ai-workflow-skills}"
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

  Downloads a prebuilt release tarball for your platform:
  Linux x86_64/arm64, macOS x86_64/arm64 (Apple Silicon).
  For anything else, use 'cargo install --git ...' (see README).

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
  local artifact
  artifact="$(find "${tmp}" -type f -name "${PKG_NAME}" -print -quit)"
  [[ -n "${artifact}" ]] && [[ -f "${artifact}" ]] || {
    echo -e "${RED}Binary not found in tarball (expected a file named ${PKG_NAME}).${NC}" >&2
    rm -rf "${tmp}"
    exit 1
  }
  install -m 0755 "${artifact}" "${BIN_DIR}/${PKG_NAME}"
  rm -rf "${tmp}"
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
PLATFORM=""

case "${OS}-${ARCH}" in
  Linux-x86_64) PLATFORM="linux-x86_64" ;;
  Linux-aarch64|Linux-arm64) PLATFORM="linux-arm64" ;;
  Darwin-x86_64) PLATFORM="macos-x86_64" ;;
  Darwin-arm64) PLATFORM="macos-arm64" ;;
  *)
    echo -e "${RED}No prebuilt tarball for ${OS} ${ARCH}. Use 'cargo install --git ...' instead (see README).${NC}" >&2
    exit 1
    ;;
esac

command -v curl >/dev/null || {
  echo -e "${RED}curl is required.${NC}" >&2
  exit 1
}
command -v python3 >/dev/null || {
  echo -e "${RED}python3 is required to resolve latest release.${NC}" >&2
  exit 1
}

VER_RESOLVED="$(resolve_version)"
echo -e "${BLUE}Version: ${VER_RESOLVED}${NC}"

download_binary "${VER_RESOLVED}"

ensure_env_snippet

echo ""
echo -e "${GREEN}Done.${NC}"
echo -e "${BLUE}Interactive UI:${NC} ${BIN_DIR}/${PKG_NAME}"
