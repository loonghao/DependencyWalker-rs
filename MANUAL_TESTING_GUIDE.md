# DependencyWalker RS - Manual Testing Guide (Slint UI)

## 🎯 GUI Fixes Testing Instructions

This guide provides step-by-step instructions to manually test the three GUI fixes implemented in DependencyWalker RS using the Slint UI framework.

## 🚀 Getting Started

### Prerequisites
- Windows operating system
- Rust toolchain installed
- DependencyWalker RS source code

### Running the Application

#### Use Slint GUI (default with fixes)
```bash
# Default behavior - uses Slint with all fixes applied
cargo run --bin depwalker
```

**Note**: All fixes are implemented in the Slint UI version, which is the default.

## 🧪 Test Cases

### Test 1: Drag-and-Drop Functionality

**Objective**: Verify that drag-and-drop works correctly with proper visual feedback and file validation.

**Steps**:
1. Start the application with Slint GUI (default)
2. Open Windows File Explorer
3. Navigate to a folder containing DLL or EXE files
4. **Test Valid Files**:
   - Drag a `.exe` file onto the application window
   - Observe visual feedback during drag (border should highlight)
   - Verify file is accepted and analysis begins
   - Repeat with `.dll`, `.sys`, `.ocx`, and `.mll` files

5. **Test Invalid Files**:
   - Drag a `.txt` file onto the application window
   - Verify file is rejected with appropriate error message
   - Test with other non-PE files (`.png`, `.doc`, etc.)

**Expected Results**:
- ✅ Visual feedback during drag operation (highlighted border)
- ✅ Valid PE files are accepted and analyzed
- ✅ Invalid files are rejected with clear error messages
- ✅ Drag state resets properly after drop

### Test 2: File Browser Dialog Management

**Objective**: Ensure only one file dialog can be open at a time with proper modal behavior.

**Steps**:
1. Start the application with Slint GUI (default)
2. Click the "📂 Browse Files" button
3. **Test Single Dialog**:
   - Verify file dialog opens
   - Button should show "Opening..." text
   - Button should be disabled/non-clickable

4. **Test Multiple Dialog Prevention**:
   - While dialog is open, try clicking the button again
   - Verify no additional dialogs open
   - Button remains in "Opening..." state

5. **Test Dialog Completion**:
   - Select a valid PE file in the dialog
   - Verify dialog closes
   - Button returns to "📂 Browse Files" text
   - Button becomes clickable again

6. **Test Dialog Cancellation**:
   - Open dialog again
   - Cancel the dialog (press Escape or click Cancel)
   - Verify button state resets properly

**Expected Results**:
- ✅ Only one dialog can be open at a time
- ✅ Button shows appropriate state ("Opening..." vs "📂 Browse Files")
- ✅ Button is disabled when dialog is open
- ✅ State resets properly on dialog completion or cancellation

### Test 3: Welcome Screen Layout

**Objective**: Verify that the welcome screen content is properly centered and responsive.

**Steps**:
1. Start the application with Slint GUI (default)
2. **Test Initial Layout**:
   - Verify title "DependencyWalker RS" is centered
   - Verify subtitle is centered below title
   - Verify drag zone is centered in the window
   - Verify "Browse Files" button is centered

3. **Test Responsive Design**:
   - Resize the window to different sizes (small, medium, large)
   - Verify content remains centered at all sizes
   - Verify text and elements maintain proper spacing
   - Verify drag zone adapts to window size

4. **Test Visual Hierarchy**:
   - Verify proper font sizes and colors
   - Verify spacing between elements
   - Verify file type icons are properly aligned
   - Verify supported formats list is centered

**Expected Results**:
- ✅ All content is properly centered horizontally and vertically
- ✅ Layout adapts to different window sizes
- ✅ Visual hierarchy is clear and consistent
- ✅ Spacing and alignment are proper

## 🔍 Additional Testing

### Error Handling
1. Test with corrupted PE files
2. Test with very large files
3. Test with files that require elevated permissions

### Performance
1. Test with multiple rapid drag-and-drop operations
2. Test dialog opening/closing rapidly
3. Test window resizing performance

### Edge Cases
1. Test with files that have no extension
2. Test with files that have multiple extensions
3. Test with very long file paths

## 📋 Test Results Checklist

Use this checklist to track your testing progress:

### Drag-and-Drop
- [ ] Visual feedback during drag
- [ ] Valid files accepted (.exe, .dll, .sys, .ocx, .mll)
- [ ] Invalid files rejected
- [ ] Error messages displayed correctly
- [ ] Drag state resets properly

### File Dialog
- [ ] Single dialog enforcement
- [ ] Button state changes correctly
- [ ] Dialog completion resets state
- [ ] Dialog cancellation resets state
- [ ] Modal behavior works

### Layout
- [ ] Content centered initially
- [ ] Responsive to window resizing
- [ ] Visual hierarchy maintained
- [ ] Proper spacing and alignment
- [ ] File type icons aligned

## 🐛 Reporting Issues

If you find any issues during testing:

1. Note the specific test case and steps
2. Record the expected vs actual behavior
3. Include screenshots if applicable
4. Note your system configuration
5. Check console output for error messages

## ✅ Success Criteria

All tests pass when:
- Drag-and-drop works smoothly with proper feedback
- File dialogs behave modally without conflicts
- Welcome screen layout is centered and responsive
- No crashes or unexpected behavior occurs
- Error messages are clear and helpful

## 🎉 Conclusion

These fixes significantly improve the user experience of DependencyWalker RS by providing:
- Intuitive drag-and-drop functionality
- Professional dialog management
- Clean, centered layout design

The application is now ready for production use with these enhancements!
