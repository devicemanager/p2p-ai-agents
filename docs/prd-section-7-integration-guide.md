# PRD Section 7 Integration Guide

## Overview

This document explains how to integrate the new Section 7 ("Project Type Specific Requirements") into the existing PRD.

---

## What to Replace/Insert

### Current PRD Status

**Location:** `docs/PRD.md`

**Current Structure:**
```
1. Product Vision & Strategy
2. User Personas
3. Core Features & Requirements
4. Technical Architecture
5. Non-Functional Requirements
6. Dependencies & Constraints
7. Success Metrics & KPIs           ← Currently labeled as "7"
8. Risks & Mitigation
9. Release Plan
10. Documentation Requirements
11. Open Questions
12. Appendices
```

### New Section 7 Details

The new "Project Type Specific Requirements" section needs to be **inserted between Section 6 (Dependencies & Constraints) and the current Section 7 (Success Metrics & KPIs)**.

**New Structure (after insertion):**
```
1. Product Vision & Strategy
2. User Personas
3. Core Features & Requirements
4. Technical Architecture
5. Non-Functional Requirements
6. Dependencies & Constraints
7. Project Type Specific Requirements    ← NEW SECTION
8. Success Metrics & KPIs                ← Renumbered from 7
9. Risks & Mitigation                    ← Renumbered from 8
10. Release Plan                         ← Renumbered from 9
11. Documentation Requirements          ← Renumbered from 10
12. Open Questions                      ← Renumbered from 11
13. Appendices                          ← Renumbered from 12
```

---

## How to Integrate

### Option A: Direct Insertion (Recommended)

1. **Open the main PRD file:**
   ```bash
   open docs/PRD.md
   ```

2. **Find the location to insert** (after Dependencies & Constraints section):
   - Search for: `## 6. Dependencies & Constraints`
   - Scroll to the end of that section
   - The new Section 7 content starts right after

3. **Copy the full Section 7 content:**
   ```bash
   # Read the generated section
   cat docs/prd-section-7-project-type-specific-requirements.md
   ```

4. **Insert after Section 6, before current Section 7:**
   - In your editor (VS Code, etc.), place cursor at end of Section 6
   - Paste the entire Section 7 content
   - Verify the "---" dividers are correct

5. **Renumber all subsequent sections:**
   - Find: `## 7. Success Metrics & KPIs` → Replace with: `## 8. Success Metrics & KPIs`
   - Find: `## 8. Risks & Mitigation` → Replace with: `## 9. Risks & Mitigation`
   - Find: `## 9. Release Plan` → Replace with: `## 10. Release Plan`
   - Find: `## 10. Documentation Requirements` → Replace with: `## 11. Documentation Requirements`
   - Find: `## 11. Open Questions` → Replace with: `## 12. Open Questions`
   - Find: `## 12. Appendices` → Replace with: `## 13. Appendices`

6. **Update table of contents** (if present):
   - Add entry for Section 7
   - Update numbering for all downstream sections

### Option B: Automated Integration (Bash Script)

```bash
#!/bin/bash

# Backup original PRD
cp docs/PRD.md docs/PRD.md.backup

# Find the line number where Section 6 ends
INSERTION_LINE=$(grep -n "^## 7. Success Metrics" docs/PRD.md | cut -d: -f1)

# Split file at insertion point
head -n $((INSERTION_LINE - 1)) docs/PRD.md > /tmp/prd_part1.md
tail -n +$INSERTION_LINE docs/PRD.md > /tmp/prd_part2.md

# Renumber sections in part 2
sed -i 's/^## 7\. /## 8. /g' /tmp/prd_part2.md
sed -i 's/^## 8\. /## 9. /g' /tmp/prd_part2.md
sed -i 's/^## 9\. /## 10. /g' /tmp/prd_part2.md
sed -i 's/^## 10\. /## 11. /g' /tmp/prd_part2.md
sed -i 's/^## 11\. /## 12. /g' /tmp/prd_part2.md
sed -i 's/^## 12\. /## 13. /g' /tmp/prd_part2.md

# Combine with new section
cat /tmp/prd_part1.md \
    docs/prd-section-7-project-type-specific-requirements.md \
    /tmp/prd_part2.md > docs/PRD.md

# Verify result
echo "✓ Integration complete"
wc -l docs/PRD.md
```

### Option C: Keep as Separate Document (Interim)

If you want to keep Section 7 as a separate document while drafting/reviewing:

1. **Create reference in main PRD:**
   ```markdown
   ## 7. Project Type Specific Requirements
   
   **Status:** See dedicated document  
   **Location:** [docs/prd-section-7-project-type-specific-requirements.md](prd-section-7-project-type-specific-requirements.md)
   
   This section details the hybrid CLI + P2P architecture, including:
   - Command structure and CLI design
   - Configuration cascade strategy
   - libp2p protocol specifications
   - Local REST API surface
   - Key management and identity
   
   [Read Full Section 7](prd-section-7-project-type-specific-requirements.md)
   ```

2. **In GitHub/repo:**
   - Link appears as clickable reference
   - Can be reviewed separately
   - Easier to manage while still drafting

3. **Later consolidate** into main PRD when finalized

---

## Renumbering Details

### In Main Content

The following section numbers need to be updated throughout:

```markdown
# BEFORE → AFTER

## 7. Success Metrics & KPIs → ## 8. Success Metrics & KPIs
### 7.1 Technical Metrics → ### 8.1 Technical Metrics
### 7.2 User Adoption Metrics → ### 8.2 User Adoption Metrics
...

## 8. Risks & Mitigation → ## 9. Risks & Mitigation
### 8.1 Technical Risks → ### 9.1 Technical Risks
...
```

### In Cross-References

Find all internal links like `See Section 7.1` and update:

```markdown
# Examples to search/replace:
- "See section 7 for" → "See section 8 for"
- "As discussed in 7.2" → "As discussed in 8.2"
- "[Success Criteria](#7-success-metrics)" → "[Success Criteria](#8-success-metrics)"
```

### In Appendices

Update the "Version History" and "Reference Links" in Appendix 12.3:

```markdown
### 12.3 References
- [High-Level Design](high-level-design.md)
- [System Architecture](architecture/system-overview.md)
- [Agent Protocol](agent-protocol.md)
- [Project Type Requirements](prd-section-7-project-type-specific-requirements.md) ← NEW
- [Security Architecture](architecture/security.md)
```

---

## Document Organization

### After Integration, Your Docs Folder Structure:

```
docs/
├── PRD.md                                    # Main PRD (now with Section 7)
├── prd-section-7-summary.md                 # Executive summary of Section 7
├── prd-section-7-integration-guide.md        # This file
├── PRD_VALIDATION_PROCESS.md                # Existing validation docs
├── developer/
│   ├── cli-reference.md                     # Reference from Section 7
│   ├── configuration.md                     # Reference from Section 7
│   ├── protocols/                           # Reference from Section 7
│   │   ├── task-protocol.proto
│   │   ├── identity-protocol.proto
│   │   └── vector-protocol.proto
│   └── api/
│       └── rest-api-openapi.yaml
├── getting-started.md                       # Reference from Section 7
└── ...
```

---

## Validation Checklist

After integrating Section 7, verify:

- [ ] Section 7 content inserted at correct location (after Section 6)
- [ ] All subsequent sections renumbered (7 → 8, 8 → 9, etc.)
- [ ] Internal cross-references updated throughout document
- [ ] Links to external files still valid
- [ ] Markdown formatting consistent with rest of PRD
- [ ] Table of contents (if present) updated
- [ ] No duplicate content
- [ ] Line breaks/spacing maintain readability
- [ ] All diagrams render correctly in Markdown
- [ ] No merge conflicts or broken references

### Testing Commands

```bash
# Check for any remaining old section numbers that should be renumbered
grep -n "^## 7\." docs/PRD.md

# Count total sections
grep "^## [0-9]\." docs/PRD.md | wc -l
# Should output: 13 (was 12, now +1)

# Verify file integrity
wc -l docs/PRD.md
# Should be significantly larger than before

# Search for broken links
grep -o "\[.*\](#[0-9]" docs/PRD.md | sort | uniq
# Verify all section numbers match

# Validate Markdown syntax (if you have mdlint installed)
mdlint docs/PRD.md
```

---

## Timeline for Integration

### Week 1: Review & Feedback
- [ ] Share Section 7 draft with stakeholders
- [ ] Engineering Lead review
- [ ] Security Lead review
- [ ] Get approval to integrate

### Week 2: Integration
- [ ] Integrate Section 7 into main PRD
- [ ] Renumber all sections
- [ ] Test all cross-references
- [ ] Update documentation index
- [ ] Create PR for review

### Week 3: Finalization
- [ ] Merge into main branch
- [ ] Generate PDFs/exports if needed
- [ ] Update version number in PRD header
- [ ] Announce to team

---

## Template for PR Description

When creating a Pull Request to merge Section 7:

```markdown
## PR: Add Section 7 - Project Type Specific Requirements

### What
Adds comprehensive "Project Type Specific Requirements" section to PRD, detailing:
- Hybrid CLI + P2P architecture
- Configuration cascade strategy (Defaults < File < Env < CLI)
- libp2p protocol specifications
- Local REST API surface (localhost:9000)
- Zero-config Ed25519 key management

### Why
Validates and documents the technical decisions made during product scoping:
- How users interact with the system (CLI)
- How agents collaborate (P2P networking)
- How configuration is managed (cascade)
- How identity/security works (Ed25519)

### Scope
- New Section 7: ~15,000 words
- Supporting documents:
  - `prd-section-7-summary.md` (executive summary)
  - `prd-section-7-integration-guide.md` (this document)
- Updates to existing sections: Renumbering only (Sections 7-12 → 8-13)

### Validation
- [ ] Engineering Lead approval
- [ ] Security Lead approval
- [ ] All cross-references verified
- [ ] Markdown formatting consistent
- [ ] No broken links

### Files Changed
- `docs/PRD.md` (renumbered, Section 7 added)
- `docs/prd-section-7-project-type-specific-requirements.md` (new)
- `docs/prd-section-7-summary.md` (new)
- `docs/prd-section-7-integration-guide.md` (new)
```

---

## Rollback Plan

If something goes wrong during integration:

```bash
# Restore from backup
cp docs/PRD.md.backup docs/PRD.md

# Or reset from git
git checkout docs/PRD.md

# Try again
```

---

## Going Live Checklist

Once integrated and approved:

- [ ] Version number updated in PRD header
- [ ] Status changed from "Draft" to "Review" or "Approved"
- [ ] Date updated (e.g., "2026-01-10")
- [ ] Team notified via Slack/email
- [ ] Documentation site (if public) updated
- [ ] Link shared in relevant channels
- [ ] Feedback survey/channel opened

---

## FAQ: Section 7 Integration

**Q: Should I integrate into the original PRD.md or create a new file?**  
A: Create the new file first (done ✓), then integrate into main PRD.md via separate PR. This allows review before merging.

**Q: What if the existing PRD structure is different?**  
A: Adapt the insertion point based on your actual PRD. The key is: after "Dependencies & Constraints" (Section 6) and before the current "Success Metrics" section.

**Q: Do I need to update external documentation sites?**  
A: Yes, if you publish docs to readthedocs, GitHub Pages, Gitbook, etc. Update after merging.

**Q: Can I rename Section 7 to something else?**  
A: Absolutely. Common alternatives:
  - "Implementation Specification"
  - "Technical Specification"
  - "Architecture & Design"
  - "Project Requirements" (simpler name)

**Q: How do I handle references from external documents?**  
A: If you have references like "See PRD Section 7" in other docs, update after you confirm the final numbering.

---

## Contact & Support

For questions about this integration:
1. Review this document thoroughly (FAQ section above)
2. Check the main Section 7 content for clarification
3. Reach out to: [Product Manager / Engineering Lead]

---

**Document Version:** 1.0  
**Created:** 2025-01-15  
**Status:** Ready for Integration
