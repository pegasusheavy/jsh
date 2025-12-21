import { Component } from '@angular/core';

@Component({
  selector: 'app-bash-zsh',
  template: `
    <div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
      <h1 class="text-4xl font-bold mb-4">Bash/ZSH Compatibility</h1>
      <p class="text-[var(--color-text-muted)] mb-8">
        jsh is fully compatible with Bash and ZSH scripts. All standard shell constructs work as expected.
      </p>

      <!-- Variables -->
      <section class="mb-12">
        <h2 class="text-2xl font-semibold mb-4 text-[var(--color-primary)]">Variables</h2>
        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 mb-4 overflow-x-auto"><code># Assignment
name="World"
count=42

# Expansion
echo $name
echo $&#123;name&#125;

# Default values
echo $&#123;name:-default&#125;    # Use default if unset
echo $&#123;name:=default&#125;    # Set and use default
echo $&#123;name:+value&#125;      # Use value if set

# Special variables
echo $?     # Last exit status
echo $$     # Current PID
echo $!     # Last background PID
echo $#     # Number of arguments
echo $&#64;     # All arguments</code></pre>
      </section>

      <!-- Control Flow -->
      <section class="mb-12">
        <h2 class="text-2xl font-semibold mb-4 text-[var(--color-primary)]">Control Flow</h2>

        <h3 class="text-lg font-medium mb-3">if/elif/else</h3>
        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 mb-4 overflow-x-auto"><code>if [ $x -eq 1 ]; then
    echo "one"
elif [ $x -eq 2 ]; then
    echo "two"
else
    echo "other"
fi</code></pre>

        <h3 class="text-lg font-medium mb-3 mt-6">for loop</h3>
        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 mb-4 overflow-x-auto"><code>for i in 1 2 3 4 5; do
    echo "Number: $i"
done

# With glob
for file in *.txt; do
    echo "Processing: $file"
done</code></pre>

        <h3 class="text-lg font-medium mb-3 mt-6">while/until loops</h3>
        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 mb-4 overflow-x-auto"><code>while [ $count -gt 0 ]; do
    echo $count
    count=$((count - 1))
done

until [ $x -ge 10 ]; do
    x=$((x + 1))
done</code></pre>

        <h3 class="text-lg font-medium mb-3 mt-6">case statement</h3>
        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 overflow-x-auto"><code>case "$cmd" in
    start)
        start_service
        ;;
    stop|halt)
        stop_service
        ;;
    *)
        echo "Usage: $0 &#123;start|stop&#125;"
        ;;
esac</code></pre>
      </section>

      <!-- Functions -->
      <section class="mb-12">
        <h2 class="text-2xl font-semibold mb-4 text-[var(--color-primary)]">Functions</h2>
        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 mb-4 overflow-x-auto"><code># Function definition
greet() &#123;
    local name="$1"
    echo "Hello, $name!"
&#125;

# Alternative syntax
function farewell &#123;
    echo "Goodbye, $1!"
&#125;

# Call functions
greet "World"
farewell "Friend"</code></pre>
      </section>

      <!-- Pipes and Redirections -->
      <section class="mb-12">
        <h2 class="text-2xl font-semibold mb-4 text-[var(--color-primary)]">Pipes &amp; Redirections</h2>
        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 mb-4 overflow-x-auto"><code># Pipes
cat file.txt | grep pattern | sort | uniq

# Output redirection
echo "text" &gt; file.txt    # Overwrite
echo "text" &gt;&gt; file.txt   # Append

# Input redirection
sort &lt; unsorted.txt

# Stderr redirection
command 2&gt; errors.log
command 2&gt;&amp;1              # Stderr to stdout

# Here documents
cat &lt;&lt;EOF
Hello, World!
This is a here document.
EOF</code></pre>
      </section>

      <!-- Command Substitution -->
      <section class="mb-12">
        <h2 class="text-2xl font-semibold mb-4 text-[var(--color-primary)]">Command Substitution</h2>
        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 mb-4 overflow-x-auto"><code># Modern syntax (preferred)
current_dir=$(pwd)
file_count=$(ls | wc -l)

# Nested substitution
result=$(echo $(cat file.txt))</code></pre>
      </section>

      <!-- Arithmetic -->
      <section class="mb-12">
        <h2 class="text-2xl font-semibold mb-4 text-[var(--color-primary)]">Arithmetic</h2>
        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 mb-4 overflow-x-auto"><code># Arithmetic expansion
result=$((2 + 3))
result=$((x * y))
result=$((count++))

# In conditions
if [ $((x % 2)) -eq 0 ]; then
    echo "Even"
fi</code></pre>
      </section>

      <!-- Logical Operators -->
      <section>
        <h2 class="text-2xl font-semibold mb-4 text-[var(--color-primary)]">Logical Operators</h2>
        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 overflow-x-auto"><code># AND - run second if first succeeds
command1 &amp;&amp; command2

# OR - run second if first fails
command1 || command2

# Chaining
mkdir dir &amp;&amp; cd dir &amp;&amp; touch file.txt

# Background execution
long_running_command &amp;</code></pre>
      </section>
    </div>
  `
})
export class BashZshPage {}
