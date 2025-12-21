import { Component } from '@angular/core';

@Component({
  selector: 'app-ash-posix',
  template: `
    <div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
      <h1 class="text-4xl font-bold mb-4">Ash/POSIX Compatibility</h1>
      <p class="text-[var(--color-text-muted)] mb-8">
        jsh supports POSIX shell options for portable scripting and lightweight environments like BusyBox.
      </p>

      <!-- Shell Options -->
      <section class="mb-12">
        <h2 class="text-2xl font-semibold mb-4 text-[var(--color-primary)]">Shell Options</h2>
        <p class="text-[var(--color-text-muted)] mb-4">
          Control shell behavior with <code>set</code> options.
        </p>

        <div class="overflow-x-auto">
          <table class="w-full text-sm">
            <thead>
              <tr class="border-b border-[var(--color-border)]">
                <th class="text-left py-2 pr-4">Option</th>
                <th class="text-left py-2 pr-4">Long Name</th>
                <th class="text-left py-2">Description</th>
              </tr>
            </thead>
            <tbody class="text-[var(--color-text-muted)]">
              <tr class="border-b border-[var(--color-border)]">
                <td class="py-2 pr-4"><code>-e</code></td>
                <td class="py-2 pr-4">errexit</td>
                <td class="py-2">Exit immediately if a command fails</td>
              </tr>
              <tr class="border-b border-[var(--color-border)]">
                <td class="py-2 pr-4"><code>-u</code></td>
                <td class="py-2 pr-4">nounset</td>
                <td class="py-2">Error on unset variable reference</td>
              </tr>
              <tr class="border-b border-[var(--color-border)]">
                <td class="py-2 pr-4"><code>-x</code></td>
                <td class="py-2 pr-4">xtrace</td>
                <td class="py-2">Print commands before execution</td>
              </tr>
              <tr class="border-b border-[var(--color-border)]">
                <td class="py-2 pr-4"><code>-n</code></td>
                <td class="py-2 pr-4">noexec</td>
                <td class="py-2">Parse only, don't execute</td>
              </tr>
              <tr class="border-b border-[var(--color-border)]">
                <td class="py-2 pr-4"><code>-a</code></td>
                <td class="py-2 pr-4">allexport</td>
                <td class="py-2">Export all variables automatically</td>
              </tr>
              <tr class="border-b border-[var(--color-border)]">
                <td class="py-2 pr-4"><code>-C</code></td>
                <td class="py-2 pr-4">noclobber</td>
                <td class="py-2">Don't overwrite files with &gt;</td>
              </tr>
              <tr class="border-b border-[var(--color-border)]">
                <td class="py-2 pr-4"><code>-f</code></td>
                <td class="py-2 pr-4">noglob</td>
                <td class="py-2">Disable pathname expansion</td>
              </tr>
              <tr>
                <td class="py-2 pr-4"><code>-b</code></td>
                <td class="py-2 pr-4">notify</td>
                <td class="py-2">Notify of job completion immediately</td>
              </tr>
            </tbody>
          </table>
        </div>

        <h3 class="text-lg font-medium mb-3 mt-6">Usage</h3>
        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 mb-4 overflow-x-auto"><code># Enable option with -
set -e              # Enable errexit
set -o errexit      # Same, by name

# Disable option with +
set +e              # Disable errexit
set +o errexit      # Same, by name

# Multiple options
set -eu             # Enable errexit and nounset

# List all options
set -o</code></pre>
      </section>

      <!-- Positional Parameters -->
      <section class="mb-12">
        <h2 class="text-2xl font-semibold mb-4 text-[var(--color-primary)]">Positional Parameters</h2>
        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 mb-4 overflow-x-auto"><code># Set positional parameters
set -- arg1 arg2 arg3

# Access parameters
echo $1    # arg1
echo $2    # arg2
echo $3    # arg3
echo $#    # 3 (count)
echo $&#64;    # arg1 arg2 arg3 (all)</code></pre>
      </section>

      <!-- Variable Declarations -->
      <section class="mb-12">
        <h2 class="text-2xl font-semibold mb-4 text-[var(--color-primary)]">Variable Declarations</h2>
        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 mb-4 overflow-x-auto"><code># Export to environment
export PATH="/usr/local/bin:$PATH"
export MYVAR="value"

# Read-only variable
readonly VERSION="1.0.0"

# Local variable (in function)
myfunc() &#123;
    local temp="value"
    echo $temp
&#125;</code></pre>
      </section>

      <!-- Source/Dot Command -->
      <section class="mb-12">
        <h2 class="text-2xl font-semibold mb-4 text-[var(--color-primary)]">Source Command</h2>
        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 mb-4 overflow-x-auto"><code># Execute script in current shell
. /path/to/script.sh
source /path/to/script.sh

# Common use: load config
if [ -f ~/.config/myapp ]; then
    . ~/.config/myapp
fi</code></pre>
      </section>

      <!-- Test Command -->
      <section class="mb-12">
        <h2 class="text-2xl font-semibold mb-4 text-[var(--color-primary)]">Test Command</h2>
        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 mb-4 overflow-x-auto"><code># File tests
[ -f file.txt ]      # Is regular file
[ -d directory ]     # Is directory
[ -e path ]          # Exists
[ -r file ]          # Is readable
[ -w file ]          # Is writable
[ -x file ]          # Is executable

# String tests
[ -z "$var" ]        # Is empty
[ -n "$var" ]        # Is not empty
[ "$a" = "$b" ]      # Equal
[ "$a" != "$b" ]     # Not equal

# Numeric tests
[ $a -eq $b ]        # Equal
[ $a -ne $b ]        # Not equal
[ $a -lt $b ]        # Less than
[ $a -gt $b ]        # Greater than
[ $a -le $b ]        # Less or equal
[ $a -ge $b ]        # Greater or equal</code></pre>
      </section>

      <!-- POSIX Script Example -->
      <section>
        <h2 class="text-2xl font-semibold mb-4">POSIX-Compatible Script</h2>
        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 overflow-x-auto"><code>#!/usr/bin/env jsh
# Portable POSIX script

set -e   # Exit on error
set -u   # Error on unset vars

readonly SCRIPT_DIR=$(dirname "$0")
readonly VERSION="1.0.0"

usage() &#123;
    echo "Usage: $0 [-h] [-v] [file...]"
    exit 1
&#125;

while [ $# -gt 0 ]; do
    case "$1" in
        -h|--help)
            usage
            ;;
        -v|--version)
            echo $VERSION
            exit 0
            ;;
        *)
            break
            ;;
    esac
    shift
done

for file in "$&#64;"; do
    if [ -f "$file" ]; then
        echo "Processing: $file"
    else
        echo "Not found: $file" &gt;&amp;2
    fi
done</code></pre>
      </section>
    </div>
  `
})
export class AshPosixPage {}
