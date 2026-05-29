# AIMF Glossary

## A

**AAUD** — AI Audio format. WAV container with AIMF/AAUD metadata.

**AIMF** — AI Media Format. Umbrella term for AIMG/AAUD/AVID.

**AIMG** — AI Image format. PNG container with AIMF/AIMG metadata.

**AVID** — AI Video format. MP4 container with AIMF/AVID metadata.

## B

**Backward compatible** — AIMF files play in standard media players (VLC, etc.) because they're valid PNG/WAV/MP4 files.

**Batch processing** — Handling multiple files with a single command.

## C

**CBOR** — Concise Binary Object Representation. Binary serialization format more compact than JSON.

**Codec** — Software that encodes/decodes media in a specific container format.

**Container format** — File format that holds media data (e.g., PNG, WAV, MP4).

## D

**Deterministic** — Same input produces same output (important for hash verification).

## E

**Ed25519** — Elliptic curve signature scheme. Used for cryptographic signing in AIMF.

**Embed** — Process of adding AIMF metadata to a media file.

**Extract** — Process of removing AIMF metadata to recover original media.

## F

**Full verification** — Checks both hash (integrity) and signature (authenticity).

## H

**Hash** — SHA-256 digest of content. Detects tampering.

## I

**Ingest** — AIMF command to create files from JSON or raw data.

**Integrity** — Assurance that file hasn't been modified (provided by hash).

## J

**JSON ingestion** — Creating AIMF files from JSON descriptions of media.

## K

**Key pair** — Private key (signing) + public key (verification).

## M

**Metadata** — AI provenance information (model, version, timestamp, etc.).

**Modality** — Type of media: image, audio, or video.

## N

**Non-repudiation** — Signer cannot deny having signed the file.

## P

**Payload** — Original media bytes (PNG, WAV, or MP4 data).

**Payload type** — Either "encoded" (standard format) or "raw" (uncompressed).

**Prompt hash** — SHA-256 of generation prompt. Protects privacy while enabling reproducibility.

**Provenance** — Origin and history of AI-generated content.

**Public key** — Shared value that verifies signatures. Safe to distribute.

## R

**Raw ingestion** — Creating AIMF files from raw PCM audio, RGB frames, etc.

**RIFF** — Resource Interchange File Format (WAV container).

## S

**Signature** — Cryptographic proof of authenticity (Ed25519).

**Streaming** — Processing files without loading entirely into memory (planned).

## T

**tEXt chunk** — PNG metadata chunk type used for AIMG.

**Timestamp** — Unix seconds when file was created. Stored in metadata.

## U

**UUID box** — MP4 container box type used for AVID.

## V

**Verification** — Process of checking hash and (if present) signature.

## W

**WAV** — Waveform Audio File Format. Base container for AAUD.