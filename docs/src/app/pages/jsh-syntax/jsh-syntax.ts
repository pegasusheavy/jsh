import { Component } from '@angular/core';

@Component({
  selector: 'app-jsh-syntax',
  template: `
    <div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
      <h1 class="text-4xl font-bold mb-4">jsh Enhanced Syntax</h1>
      <p class="text-[var(--color-text-muted)] mb-8">
        jsh extends traditional shell syntax with modern, readable constructs inspired by Rust.
      </p>

      <!-- Match Expression -->
      <section class="mb-12">
        <h2 class="text-2xl font-semibold mb-4 text-[var(--color-primary)]">match - Pattern Matching</h2>
        <p class="text-[var(--color-text-muted)] mb-4">
          A powerful pattern matching expression that replaces verbose case statements.
        </p>

        <h3 class="text-lg font-medium mb-3">Basic Syntax</h3>
        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 mb-4 overflow-x-auto"><code>match $value &#123;
    pattern1 =&gt; command1
    pattern2 =&gt; command2
    * =&gt; default_command
&#125;</code></pre>

        <h3 class="text-lg font-medium mb-3 mt-6">Pattern Types</h3>
        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 mb-4 overflow-x-auto"><code>match $x &#123;
    # Literal match
    1 =&gt; echo "one"

    # Multiple patterns (OR)
    2 | 3 =&gt; echo "two or three"

    # Range pattern
    4..10 =&gt; echo "between 4 and 10"

    # Regex pattern
    /^hello/ =&gt; echo "starts with hello"

    # Glob pattern
    *.txt =&gt; echo "text file"

    # Default (wildcard)
    * =&gt; echo "anything else"
&#125;</code></pre>

        <h3 class="text-lg font-medium mb-3 mt-6">Block Bodies</h3>
        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 overflow-x-auto"><code>match $action &#123;
    deploy =&gt; &#123;
        echo "Deploying..."
        build_app
        push_to_server
        echo "Done!"
    &#125;
    * =&gt; echo "Unknown action"
&#125;</code></pre>
      </section>

      <!-- Let/Const -->
      <section class="mb-12">
        <h2 class="text-2xl font-semibold mb-4 text-[var(--color-primary)]">let/const - Variable Bindings</h2>
        <p class="text-[var(--color-text-muted)] mb-4">
          Cleaner variable declaration with optional immutability.
        </p>

        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 mb-4 overflow-x-auto"><code># Mutable variable
let name = "World"
let count = 0

# Immutable constant
const VERSION = "1.0.0"
const PI = "3.14159"

# Command substitution
let current_dir = $(pwd)
let files = $(ls -la)

# Use variables
echo "Hello, $name! Version $VERSION"</code></pre>
      </section>

      <!-- Loop -->
      <section class="mb-12">
        <h2 class="text-2xl font-semibold mb-4 text-[var(--color-primary)]">loop - Infinite Loops</h2>
        <p class="text-[var(--color-text-muted)] mb-4">
          A clean syntax for loops that run until explicitly broken.
        </p>

        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 mb-4 overflow-x-auto"><code># REPL-style loop
loop &#123;
    read -p "jsh&gt; " command

    if [ "$command" = "exit" ]; then
        break
    fi

    eval "$command"
&#125;

# Counter with break
let i = 0
loop &#123;
    i=$((i + 1))
    echo "Iteration $i"

    if [ $i -ge 5 ]; then
        break
    fi
&#125;</code></pre>
      </section>

      <!-- Try-Catch-Finally -->
      <section class="mb-12">
        <h2 class="text-2xl font-semibold mb-4 text-[var(--color-primary)]">try/catch/finally - Error Handling</h2>
        <p class="text-[var(--color-text-muted)] mb-4">
          Structured error handling with optional cleanup.
        </p>

        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 mb-4 overflow-x-auto"><code># Full try-catch-finally
try &#123;
    let result = $(risky_operation)
    process "$result"
&#125; catch error &#123;
    echo "Error occurred: $error" &gt;&amp;2
    exit 1
&#125; finally &#123;
    # Always runs
    cleanup_temp_files
    close_connections
&#125;

# Try-catch only
try &#123;
    make_api_call
&#125; catch e &#123;
    echo "API failed: $e"
&#125;</code></pre>
      </section>

      <!-- fn - Function Definition -->
      <section class="mb-12">
        <h2 class="text-2xl font-semibold mb-4 text-[var(--color-primary)]">fn - Function Shorthand</h2>
        <p class="text-[var(--color-text-muted)] mb-4">
          A concise syntax for defining functions.
        </p>

        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 mb-4 overflow-x-auto"><code># Simple function
fn greet &#123;
    echo "Hello, $1!"
&#125;

# Function with logic
fn is_even &#123;
    let num = $1
    if [ $((num % 2)) -eq 0 ]; then
        return 0
    else
        return 1
    fi
&#125;

# Call functions
greet "World"
if is_even 4; then
    echo "4 is even"
fi</code></pre>

        <div class="bg-[var(--color-surface)] rounded-lg p-4 mt-4 border border-[var(--color-border)]">
          <p class="text-sm text-[var(--color-text-muted)]">
            <strong>Note:</strong> <code>fn</code> is equivalent to the traditional
            <code>name() &#123; &#125;</code> syntax but provides a cleaner, more modern look.
          </p>
        </div>
      </section>

      <!-- Named Function Parameters -->
      <section class="mb-12">
        <h2 class="text-2xl font-semibold mb-4 text-[var(--color-primary)]">Named Function Parameters</h2>
        <p class="text-[var(--color-text-muted)] mb-4">
          jsh introduces named parameters for functions, making code more readable than positional arguments.
        </p>

        <h3 class="text-lg font-medium mb-3">Basic Syntax</h3>
        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 mb-4 overflow-x-auto"><code># Define function with named parameters
greet(name, greeting) &#123;
    echo "$greeting, $name!"
&#125;

# Call the function
greet "World" "Hello"    # Output: Hello, World!
greet "User" "Welcome"   # Output: Welcome, User!</code></pre>

        <h3 class="text-lg font-medium mb-3 mt-6">Works with All Function Styles</h3>
        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 mb-4 overflow-x-auto"><code># Shorthand style
add(a, b) &#123;
    echo $((a + b))
&#125;

# fn keyword style
fn multiply(x, y) &#123;
    echo $((x * y))
&#125;

# function keyword style
function divide(num, denom) &#123;
    echo $((num / denom))
&#125;

# Call them
add 10 20        # Output: 30
multiply 6 7     # Output: 42
divide 100 5     # Output: 20</code></pre>

        <h3 class="text-lg font-medium mb-3 mt-6">Compatibility with Positional Parameters</h3>
        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 mb-4 overflow-x-auto"><code># Named parameters work alongside $1, $2, etc.
show(first, second) &#123;
    echo "Named: first=$first second=$second"
    echo "Positional: arg1=$1 arg2=$2"
&#125;

show "foo" "bar"
# Output:
# Named: first=foo second=bar
# Positional: arg1=foo arg2=bar</code></pre>

        <div class="bg-[var(--color-surface)] rounded-lg p-4 mt-4 border border-[var(--color-border)]">
          <p class="text-sm text-[var(--color-text-muted)]">
            <strong>Tip:</strong> Named parameters make your code self-documenting. Instead of
            <code>process_file $1 $2 $3</code>, you can write
            <code>process_file(input, output, format)</code> to clearly show what each argument means.
          </p>
        </div>
      </section>

      <!-- Comparison -->
      <section>
        <h2 class="text-2xl font-semibold mb-4">Comparison with Traditional Syntax</h2>
        <div class="grid md:grid-cols-2 gap-4">
          <div>
            <h3 class="font-medium mb-2 text-[var(--color-text-muted)]">Traditional Bash</h3>
            <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 text-sm overflow-x-auto"><code>case "$cmd" in
    start)
        start_service
        ;;
    stop)
        stop_service
        ;;
    *)
        echo "Unknown"
        ;;
esac</code></pre>
          </div>
          <div>
            <h3 class="font-medium mb-2 text-[var(--color-primary)]">jsh Enhanced</h3>
            <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 text-sm overflow-x-auto"><code>match $cmd &#123;
    start =&gt; start_service
    stop =&gt; stop_service
    * =&gt; echo "Unknown"
&#125;</code></pre>
          </div>
        </div>
      </section>
    </div>
  `
})
export class JshSyntaxPage {}
