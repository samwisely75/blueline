# Agent Command Migration Guide

## Overview

This guide provides step-by-step instructions for agents performing command system migrations using the new **Dynamic Command Discovery System**. This system enables **zero-conflict parallel development** for unlimited agents working simultaneously.

## 🔧 Prerequisites

### Required Reading (CRITICAL)

**Agents MUST read and strictly follow these documents before starting:**

1. **`docs/DEV_GUIDE.md`** - Core development guidelines and architecture
2. **`docs/DEV_WORKFLOW.md`** - Complete development workflow and process  
3. **`docs/DEV_CODING.md`** - Coding standards and formatting requirements
4. **`SESSION_NOTES.md`** - Current session context and previous work

### System Requirements

- Temporary repository clone (NOT primary repo)
- Git configured with proper credentials
- Rust toolchain with `cargo` available
- Access to GitHub CLI (`gh`) for PR management

### Step 1: Environment Setup

```bash
# 1. Clone to temporary location using proper directory structure
cd ~/Sources/samwisely75/rust/temp
git clone https://github.com/samwisely75/blueline.git blueline-dev-[ISSUE_NUMBER]
cd blueline-dev-[ISSUE_NUMBER]

# 2. Checkout develop branch and create feature branch
git checkout develop
git pull origin develop
git checkout -b feature/[command-name]-command-[issue-number]

# 3. Verify you're in the right location
pwd  # Should show: ~/Sources/samwisely75/rust/temp/blueline-dev-[ISSUE_NUMBER]
```

### Step 2: Analysis and Planning

### Step 3: Implementation

### Step 4: Quality Assurance (MANDATORY)

### Step 5: Commit and PR Creation
