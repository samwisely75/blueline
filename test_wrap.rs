// Simple test script to verify line wrapping
use blueline::repl::display_cache::debug_wrap_line;

fn main() {
    println!("Testing line wrapping...");
    
    // Test case 1: Short line
    let short_line = "GET /api/users";
    println!("\n=== TEST 1: Short line ===");
    debug_wrap_line(short_line, 80);
    
    // Test case 2: Long line that should wrap
    let long_line = "This is a very long line that should definitely wrap across multiple segments when the content width is small enough to force wrapping behavior";
    println!("\n=== TEST 2: Long line with width 30 ===");
    debug_wrap_line(long_line, 30);
    
    // Test case 3: Very long line like you might have on line 30
    let very_long_line = "POST /api/users HTTP/1.1\nHost: example.com\nContent-Type: application/json\nAuthorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
    println!("\n=== TEST 3: Very long line with width 80 ===");
    debug_wrap_line(very_long_line, 80);
    
    println!("\nDone!");
}
