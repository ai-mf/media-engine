# Best Practices for AIMF

## Key Management

### DO ✅

```bash
# Generate one key per identity
cargo run --bin aimf -- gen-key --output my_ai_identity.key

# Store in secure location
mkdir -p ~/.secure/keys/
mv my_ai_identity.key ~/.secure/keys/
chmod 600 ~/.secure/keys/my_ai_identity.key

# Set environment variable for scripts
export AIMF_SIGNING_KEY=~/.secure/keys/my_ai_identity.key

# Backup encrypted
gpg --encrypt --recipient your@email.com ~/.secure/keys/my_ai_identity.key

DON'T ❌
bash

# Never commit private keys
git add private.key  # DANGER!

# Never use default names in production
cp private.key /tmp/  # DANGER!

# Never share via chat/email
cat private.key | pbcopy  # DANGER!

Metadata Quality
Good Metadata
json

{
  "model_name": "StableDiffusion-v1.5-Finetuned-Cats",
  "model_version": "2024-01-15",
  "modality": "image",
  "format": "image/png",
  "timestamp": 1705315200
}

Bad Metadata
json

{
  "model_name": "model",        // Too vague
  "model_version": "latest",    // Not reproducible
  "modality": "ai",             // Not specific
  "timestamp": 0                // No date
}

Prompt Hashing
When to include prompt hash

✅ DO include:

    Research reproducibility

    Training data provenance

    Legal/audit requirements

❌ DON'T include:

    User-generated content (privacy)

    Commercial prompts (IP protection)

    When prompts are very large

Hashing prompts
bash

# Client-side: hash before sending
echo "a beautiful sunset over mountains" | sha256sum
# 5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8

# Include hash in creation
echo '{...}' | aimf json --prompt-hash 5e8848...
echo '{...}' | aimf raw --prompt-hash 5e8848...

Batch Processing
Efficient batch verification
bash

# Good: Parallel processing
find . -name "*.aimg" -print0 | xargs -0 -P 4 -I {} aimf verify {}

# Better: Use built-in batch
aimf batch --input "*.aimg" --verify --parallel --jobs 4

# Best: With progress and logging
aimf batch --input "*.aimg" --verify --parallel --verbose 2>&1 | tee verify.log

Memory management for large files
bash

# Process one at a time (low memory)
for file in *.avid; do
    aimf verify "$file"
done

# Use streaming when available (future)
# aimf verify --stream large_file.avid

Integration Patterns
Web Service
rust

// Verify before processing
async fn upload_handler(file: Vec<u8>) -> Result<()> {
    let container = extract_aimg_from_png(&file)?;
    
    match container.full_verify() {
        Ok(true) => {
            // Process verified content
            process_payload(container.payload)
        }
        Ok(false) => Err("Invalid signature".into()),
        Err(_) => Err("Corrupted file".into()),
    }
}