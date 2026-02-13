# Documentation Index

Complete guide to `spatial-maker` documentation.

## Quick Links

- **[README.md](./README.md)** — Start here! Main project overview
- **[QUICK_START.md](./QUICK_START.md)** — ⚡ Quick reference guide (5 minutes)
- **[WHATS_NEW.md](./WHATS_NEW.md)** — New automatic format conversion feature
- **[USAGE.md](./USAGE.md)** — Complete usage guide with examples

## 🎉 v0.1.0 Release Documentation

- **[RELEASE_PACKAGE.md](./RELEASE_PACKAGE.md)** — 📦 Complete release manifest
- **[RELEASE_NOTES_v0.1.0.md](./RELEASE_NOTES_v0.1.0.md)** — 📋 Comprehensive release notes
- **[RELEASE_SUMMARY.md](./RELEASE_SUMMARY.md)** — 📊 Executive summary
- **[BUILD_VERIFICATION_REPORT.md](./BUILD_VERIFICATION_REPORT.md)** — ✅ Build/test verification
- **[QUICK_START.md](./QUICK_START.md)** — 🚀 Developer quick reference

## Feature Documentation

### Automatic Format Conversion (New!)
- **[AUTOMATIC_CONVERSION.md](./AUTOMATIC_CONVERSION.md)** — Complete guide to automatic AVIF/JXL/HEIC conversion
  - Overview and requirements
  - Installation instructions (ffmpeg)
  - Usage examples (CLI and library)
  - How it works internally
  - Conversion parameters and quality
  - Error handling and troubleshooting
  - Performance metrics
  - FAQ

- **[WHATS_NEW.md](./WHATS_NEW.md)** — User-friendly summary of the new feature
  - Quick start
  - Before/after comparison
  - Real-world use cases
  - FAQ
  - Supported formats

- **[IMPLEMENTATION_SUMMARY.md](./IMPLEMENTATION_SUMMARY.md)** — Technical implementation details
  - Architecture and design decisions
  - Code changes and testing
  - Performance characteristics
  - Security considerations
  - Future enhancements

- **[NATIVE_DECODING_PROPOSAL.md](./NATIVE_DECODING_PROPOSAL.md)** — Native decoder design document
  - Design rationale for AVIF/JXL/HEIC native support
  - Feature flag architecture
  - Fallback system design
  - Implementation approach

### Output Formats
- **[OUTPUT_FORMATS.md](./OUTPUT_FORMATS.md)** — Guide to output formats and naming
  - Stereo pair formats (side-by-side, top-bottom, separate)
  - Image encoding (JPEG, PNG)
  - MV-HEVC spatial photos (.heic)
  - File naming conventions

### Image Format Support
- **[IMAGE_FORMATS.md](./IMAGE_FORMATS.md)** — Detailed format support reference
- **[IMAGE_FORMAT_SUPPORT.md](./IMAGE_FORMAT_SUPPORT.md)** — Format compatibility matrix
- **[FORMAT_ENHANCEMENT_SUMMARY.md](./FORMAT_ENHANCEMENT_SUMMARY.md)** — Format support enhancements

## Testing & Examples

- **[EXAMPLE_OUTPUTS.md](./EXAMPLE_OUTPUTS.md)** — Example output files and verification
  - Sample input: `example-humanos.jpg`
  - Generated outputs (stereo, spatial .heic)
  - Metadata verification with `spatial info`

- **[TEST_COMPLETION_REPORT.md](./TEST_COMPLETION_REPORT.md)** — Detailed testing report
  - Test coverage
  - Bug fixes
  - Integration testing results

- **[TESTING_COMPLETE.md](./TESTING_COMPLETE.md)** — Testing summary

## Project Status

- **[PROJECT_STATUS.md](../PROJECT_STATUS.md)** — Overall project status and roadmap
- **[CHANGELOG.md](../CHANGELOG.md)** — Version history and changes

## For Contributors

- **[CONTRIBUTING.md](../CONTRIBUTING.md)** — How to contribute
- **[CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md)** — Community guidelines
- **[SECURITY.md](../SECURITY.md)** — Security policy
- **[LICENSE](../LICENSE)** — MIT License

## By Use Case

### Getting Started
1. [README.md](./README.md) — Project overview
2. [QUICK_START.md](./QUICK_START.md) — Quick reference (start here!)
3. [USAGE.md](./USAGE.md) — How to use the CLI and library
4. [WHATS_NEW.md](./WHATS_NEW.md) — New features

### v0.1.0 Release Information
1. [RELEASE_PACKAGE.md](./RELEASE_PACKAGE.md) — Complete release manifest
2. [RELEASE_NOTES_v0.1.0.md](./RELEASE_NOTES_v0.1.0.md) — Full release notes
3. [RELEASE_SUMMARY.md](./RELEASE_SUMMARY.md) — Executive summary
4. [BUILD_VERIFICATION_REPORT.md](./BUILD_VERIFICATION_REPORT.md) — Test/build results

### Working with HEIC/AVIF/JXL Images
1. [WHATS_NEW.md](./WHATS_NEW.md) — Quick overview
2. [AUTOMATIC_CONVERSION.md](./AUTOMATIC_CONVERSION.md) — Complete guide
3. [NATIVE_DECODING_PROPOSAL.md](./NATIVE_DECODING_PROPOSAL.md) — Native decoder design
4. [IMPLEMENTATION_SUMMARY.md](./IMPLEMENTATION_SUMMARY.md) — Technical details

### Understanding Output Formats
1. [OUTPUT_FORMATS.md](./OUTPUT_FORMATS.md) — Format guide
2. [EXAMPLE_OUTPUTS.md](./EXAMPLE_OUTPUTS.md) — Real examples

### Troubleshooting
1. [AUTOMATIC_CONVERSION.md](./AUTOMATIC_CONVERSION.md#troubleshooting) — Common issues
2. [USAGE.md](./USAGE.md) — Usage patterns
3. [EXAMPLE_OUTPUTS.md](./EXAMPLE_OUTPUTS.md) — Expected outputs

### Development & Testing
1. [BUILD_VERIFICATION_REPORT.md](./BUILD_VERIFICATION_REPORT.md) — Complete build/test verification
2. [IMPLEMENTATION_SUMMARY.md](./IMPLEMENTATION_SUMMARY.md) — Architecture
3. [TEST_COMPLETION_REPORT.md](./TEST_COMPLETION_REPORT.md) — Testing details
4. [CONTRIBUTING.md](../CONTRIBUTING.md) — Contribution guide

## Quick Reference

### Supported Input Formats
**Native (no conversion):**
- JPEG, PNG, GIF, BMP, TIFF, WebP

**Auto-converted (requires ffmpeg):**
- AVIF, JPEG XL (JXL), HEIC/HEIF

### Output Formats
**Stereo:**
- Side-by-Side (SBS) — `.jpg` or `.png`
- Top-and-Bottom (TB) — `.jpg` or `.png`
- Separate Files — `_L.jpg` and `_R.jpg`

**Spatial:**
- MV-HEVC (.heic) — Apple spatial photo format

### Common Commands

```bash
# Process JPEG/PNG (native)
cargo run --example photo -- --input photo.jpg --output spatial.jpg

# Process HEIC (auto-converted)
cargo run --example photo -- --input photo.heic --output spatial.jpg

# Generate MV-HEVC spatial photo
cargo run --example photo -- --input photo.jpg --output spatial.heic --mvhevc

# Top-bottom stereo
cargo run --example photo -- --input photo.jpg --output stereo-tb.jpg --format top-bottom

# High quality JPEG output
cargo run --example photo -- --input photo.jpg --output spatial.jpg --quality 98
```

## Documentation Organization

```
spatial-maker/
├── README.md                           ⭐ Start here
├── QUICK_START.md                      ⚡ Quick reference (NEW!)
├── DOCUMENTATION_INDEX.md              📚 This file
├── RELEASE_PACKAGE.md                  📦 v0.1.0 release manifest (NEW!)
├── RELEASE_NOTES_v0.1.0.md             📋 Complete release notes (NEW!)
├── RELEASE_SUMMARY.md                  📊 Release summary (NEW!)
├── BUILD_VERIFICATION_REPORT.md        ✅ Build/test verification (NEW!)
├── WHATS_NEW.md                        ✨ New features
├── AUTOMATIC_CONVERSION.md             🔄 Auto-conversion guide
├── NATIVE_DECODING_PROPOSAL.md         🎯 Native decoder design (NEW!)
├── IMPLEMENTATION_SUMMARY.md           🛠️ Technical details
├── USAGE.md                            📖 Usage guide
├── OUTPUT_FORMATS.md                   📁 Output formats
├── EXAMPLE_OUTPUTS.md                  🖼️ Example results
├── IMAGE_FORMATS.md                    🎨 Format reference
├── IMAGE_FORMAT_SUPPORT.md             ✅ Format matrix
├── FORMAT_ENHANCEMENT_SUMMARY.md       📊 Format enhancements
├── TEST_COMPLETION_REPORT.md           🧪 Test report
├── TESTING_COMPLETE.md                 ✔️ Test summary
├── FEATURE_COMPLETE.md                 🎉 Feature completion
└── src/                                💻 Source code
    ├── lib.rs                          Public API
    ├── image_loader.rs                 Image loading & conversion
    ├── depth.rs                        Depth estimation
    ├── stereo.rs                       Stereo generation
    ├── model.rs                        Model management
    ├── output.rs                       Output handling
    └── error.rs                        Error types
```

## Need Help?

- **Quick start**: Read [QUICK_START.md](./QUICK_START.md) ⚡
- **Release info**: Check [RELEASE_PACKAGE.md](./RELEASE_PACKAGE.md) 📦
- **Usage examples**: See [USAGE.md](./USAGE.md)
- **New features**: Check [WHATS_NEW.md](./WHATS_NEW.md)
- **Format conversion**: Read [AUTOMATIC_CONVERSION.md](./AUTOMATIC_CONVERSION.md)
- **Native decoders**: See [NATIVE_DECODING_PROPOSAL.md](./NATIVE_DECODING_PROPOSAL.md)
- **Build/test results**: Review [BUILD_VERIFICATION_REPORT.md](./BUILD_VERIFICATION_REPORT.md)
- **Troubleshooting**: Check the FAQ sections in relevant docs
- **Contributing**: See [CONTRIBUTING.md](../CONTRIBUTING.md)

## Document Summaries

| Document | Purpose | Audience | Length |
|----------|---------|----------|--------|
| README.md | Project overview | Everyone | Medium |
| QUICK_START.md | Quick reference | Developers | Short |
| RELEASE_PACKAGE.md | Release manifest | Maintainers | Long |
| RELEASE_NOTES_v0.1.0.md | Complete release notes | Everyone | Long |
| RELEASE_SUMMARY.md | Release summary | Everyone | Medium |
| BUILD_VERIFICATION_REPORT.md | Build/test verification | Developers | Long |
| WHATS_NEW.md | New feature summary | Users | Short |
| AUTOMATIC_CONVERSION.md | Conversion guide | Users | Long |
| NATIVE_DECODING_PROPOSAL.md | Native decoder design | Developers | Long |
| IMPLEMENTATION_SUMMARY.md | Technical details | Developers | Long |
| USAGE.md | Usage guide | Users | Medium |
| OUTPUT_FORMATS.md | Output reference | Users | Medium |
| EXAMPLE_OUTPUTS.md | Example results | Users | Short |
| TEST_COMPLETION_REPORT.md | Test details | Developers | Long |

## Version Information

- **Project Version**: v0.1.0
- **Documentation Version**: Latest
- **Last Updated**: 2024

## Feedback

Found an issue with the documentation? Please:
1. Check if it's covered in another document
2. Review the FAQ sections
3. Open an issue with details
4. Suggest improvements via PR

---

**Navigation**: 
- **New users**: [README.md](./README.md) → [QUICK_START.md](./QUICK_START.md) → [USAGE.md](./USAGE.md)
- **Release info**: [RELEASE_PACKAGE.md](./RELEASE_PACKAGE.md) → [RELEASE_NOTES_v0.1.0.md](./RELEASE_NOTES_v0.1.0.md)
- **Developers**: [QUICK_START.md](./QUICK_START.md) → [BUILD_VERIFICATION_REPORT.md](./BUILD_VERIFICATION_REPORT.md)