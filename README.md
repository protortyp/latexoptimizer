# `lo`

Latexoptimizer (`lo`) helps you speeds up your local latex compilation while writing. Large figures slow you down. `lo` replaces all figures with symbolic links to a tiny placeholder image. Every now and then when you want to render the actual project, you can easily switch to the actual figures and recompile.

## Getting Started

- clone this repo and run `cargo install --path .` 
- `lo init` creates a hidden folder, finds all image files in your latex project, and moves them there. It then creates a placeholder image and symlinks all original filenames to that placeholder image
- `lo update` run this whenever you add a new image file. it will add it to its hidden collection and replace it with a placeholder
- `lo switch` removes the symlinks and replaces them with the actual files, and vice verca

## Git Pre-Commit

Add the following in `.git/hooks/pre-commit` if you're tracking your project using git.

```bash
#!/bin/bash
is_image_file() {
    case "$(lowercase "${1##*.}")" in
        png|jpg|jpeg|gif|bmp) return 0 ;;
        *) return 1 ;;
    esac
}

lowercase() {
    echo "$1" | tr '[:upper:]' '[:lower:]'
}

# gets all figures in the repo
image_files=$(git ls-files | while read -r file; do
    if is_image_file "$file"; then
        echo "$file"
    fi
done)

# check if each file is a regular file (not a symlink)
for file in $image_files; do
    if [ -L "$file" ]; then
        echo "Error: Please run 'lo switch' to replace symlinks with actual files before committing."
        exit 1
    fi
done

echo "All figure files are symlinks. Proceeding with commit."
exit 0
```
