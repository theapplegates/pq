import cloudinary
import cloudinary.uploader
import cloudinary.api
import os
import json
from dotenv import load_dotenv

# Load environment variables from .env file
load_dotenv()

# Configure Cloudinary
cloudinary.config(
    cloud_name=os.getenv('CLOUDINARY_CLOUD_NAME'),
    api_key=os.getenv('CLOUDINARY_API_KEY'),
    api_secret=os.getenv('CLOUDINARY_API_SECRET')
)

def upload_image(image_path, output_json="cloudinary_result.json", alt_text=None, aspect_ratio=None):
    """
    Upload image with enhanced responsive breakpoints and better configuration
    
    Args:
        image_path: Path to the image file
        output_json: Output JSON filename
        alt_text: Custom alt text for the image
        aspect_ratio: Optional aspect ratio for consistent sizing (e.g., "16:9", "4:3")
    """
    print(f"Uploading {image_path} to Cloudinary...")
    
    # Enhanced transformation with better quality settings
    transformation = {
        "crop": "limit",
        "width": "auto",
        "dpr": "auto",
        "gravity": "auto",
        "quality": "auto:best",  # Better quality setting
        "fetch_format": "auto"   # Let Cloudinary choose best format
    }
    
    # Add aspect ratio if specified
    if aspect_ratio:
        transformation.update({
            "crop": "fill",
            "aspect_ratio": aspect_ratio
        })
    
    result = cloudinary.uploader.upload(
        image_path,
        responsive_breakpoints={
            "create_derived": True,
            "bytes_step": 15000,  # Smaller step for more granular control
            "min_width": 200,
            "max_width": 2400,    # Increased for 4K displays
            "max_images": 8,      # More breakpoints for better optimization
            "transformation": transformation
        },
        # Additional upload parameters for better optimization
        eager=[
            {"width": 400, "height": 300, "crop": "fill", "gravity": "auto", "quality": "auto:best", "format": "webp"},
            {"width": 800, "height": 600, "crop": "fill", "gravity": "auto", "quality": "auto:best", "format": "avif"}
        ]
    )
    
    # Print enhanced summary
    print("\n=== CLOUDINARY UPLOAD SUMMARY ===")
    print(f"Original image: {result['original_filename']}.{result['format']}")
    print(f"Public ID: {result['public_id']}")
    print(f"Format: {result['format']}")
    print(f"Dimensions: {result['width']}x{result['height']} pixels")
    print(f"File size: {result['bytes']/1024:.1f} KB")
    print(f"Aspect ratio: {result['width']/result['height']:.2f}:1")
    print(f"URL: {result['secure_url']}")
    
    # Generate multiple HTML variants
    html_outputs = {
        'picture': generate_picture_html(result, alt_text),
        'img_srcset': generate_img_srcset_html(result, alt_text),
        'css_responsive': generate_css_responsive(result)
    }
    
    print(f"\n=== RESPONSIVE HTML OUTPUTS ===")
    for variant, html in html_outputs.items():
        print(f"\n--- {variant.upper().replace('_', ' ')} ---")
        print(html[:200] + "..." if len(html) > 200 else html)
    
    # Save results
    with open(output_json, 'w') as f:
        json.dump(result, f, indent=2)
    
    # Save HTML variants
    base_name = output_json.replace('.json', '')
    for variant, html in html_outputs.items():
        filename = f"{base_name}_{variant}.html"
        with open(filename, 'w') as f:
            f.write(create_complete_html(html, result, variant))
        print(f"{variant} HTML saved to {filename}")
    
    return result

def generate_picture_html(result, alt_text=None):
    """Generate optimized HTML picture element with modern formats"""
    
    responsive_breakpoints = result.get('responsive_breakpoints', [])
    if not responsive_breakpoints or len(responsive_breakpoints) == 0:
        return f'<img src="{result["secure_url"]}" alt="{alt_text or result["original_filename"]}" loading="lazy">'
    
    base_breakpoints = responsive_breakpoints[0].get('breakpoints', [])
    if not base_breakpoints:
        return f'<img src="{result["secure_url"]}" alt="{alt_text or result["original_filename"]}" loading="lazy">'
    
    # Remove duplicates and sort
    unique_breakpoints = {}
    for bp in base_breakpoints:
        width = bp['width']
        if width not in unique_breakpoints:
            unique_breakpoints[width] = bp
    
    sorted_breakpoints = sorted(unique_breakpoints.values(), key=lambda x: x['width'])
    
    public_id = result['public_id']
    cloud_name = result.get('cloud_name', os.getenv('CLOUDINARY_CLOUD_NAME', 'your-cloud-name'))
    version = result.get('version', '')
    
    html_parts = ['<picture>']
    
    # Enhanced format support with better ordering
    formats = [
        {'ext': 'avif', 'mime': 'image/avif'},    # AVIF first (best compression)
        {'ext': 'webp', 'mime': 'image/webp'},    # WebP second (good support)
        {'ext': 'jpg', 'mime': 'image/jpeg'}      # JPEG fallback
    ]
    
    for format_info in formats:
        srcset_items = []
        
        for bp in sorted_breakpoints:
            base_url = f"https://res.cloudinary.com/{cloud_name}/image/upload"
            # Enhanced transformations for better optimization
            transformations = f"c_limit,dpr_auto,f_{format_info['ext']},g_auto,q_auto:best,w_{bp['width']}"
            url = f"{base_url}/{transformations}/v{version}/{public_id}"
            srcset_items.append(f"{url} {bp['width']}w")
        
        srcset = ',\n       '.join(srcset_items)
        
        html_parts.append(f'  <source type="{format_info["mime"]}"')
        html_parts.append(f'          srcset="{srcset}"')
        # Improved sizes attribute with more breakpoints
        html_parts.append(f'          sizes="(max-width: 480px) 100vw, (max-width: 768px) 90vw, (max-width: 1200px) 60vw, 50vw">')
    
    # Enhanced fallback img with proper dimensions
    original_width = result.get('width', 0)
    original_height = result.get('height', 0)
    
    html_parts.append(f'  <img src="{result["secure_url"]}"')
    html_parts.append(f'       alt="{alt_text or result["original_filename"]}"')
    html_parts.append(f'       loading="lazy"')
    html_parts.append(f'       decoding="async"')  # Async decoding for better performance
    if original_width and original_height:
        html_parts.append(f'       width="{original_width}"')
        html_parts.append(f'       height="{original_height}"')
    html_parts.append('>')
    html_parts.append('</picture>')
    
    return '\n'.join(html_parts)

def generate_img_srcset_html(result, alt_text=None):
    """Generate simple img element with srcset (alternative to picture)"""
    
    responsive_breakpoints = result.get('responsive_breakpoints', [])
    if not responsive_breakpoints:
        return f'<img src="{result["secure_url"]}" alt="{alt_text or result["original_filename"]}" loading="lazy">'
    
    base_breakpoints = responsive_breakpoints[0].get('breakpoints', [])
    if not base_breakpoints:
        return f'<img src="{result["secure_url"]}" alt="{alt_text or result["original_filename"]}" loading="lazy">'
    
    # Create srcset for WebP format
    public_id = result['public_id']
    cloud_name = result.get('cloud_name', os.getenv('CLOUDINARY_CLOUD_NAME', 'your-cloud-name'))
    version = result.get('version', '')
    
    unique_breakpoints = {}
    for bp in base_breakpoints:
        width = bp['width']
        if width not in unique_breakpoints:
            unique_breakpoints[width] = bp
    
    sorted_breakpoints = sorted(unique_breakpoints.values(), key=lambda x: x['width'])
    
    srcset_items = []
    for bp in sorted_breakpoints:
        base_url = f"https://res.cloudinary.com/{cloud_name}/image/upload"
        transformations = f"c_limit,dpr_auto,f_webp,g_auto,q_auto:best,w_{bp['width']}"
        url = f"{base_url}/{transformations}/v{version}/{public_id}"
        srcset_items.append(f"{url} {bp['width']}w")
    
    srcset = ',\n       '.join(srcset_items)
    
    html_parts = [
        f'<img srcset="{srcset}"',
        f'     sizes="(max-width: 480px) 100vw, (max-width: 768px) 90vw, (max-width: 1200px) 60vw, 50vw"',
        f'     src="{result["secure_url"]}"',
        f'     alt="{alt_text or result["original_filename"]}"',
        f'     loading="lazy"',
        f'     decoding="async">'
    ]
    
    return '\n'.join(html_parts)

def generate_css_responsive(result):
    """Generate CSS for responsive background images"""
    
    public_id = result['public_id']
    cloud_name = result.get('cloud_name', os.getenv('CLOUDINARY_CLOUD_NAME', 'your-cloud-name'))
    version = result.get('version', '')
    
    css_parts = [
        f"/* Responsive background image for {result['original_filename']} */",
        ".responsive-bg {",
        "  background-size: cover;",
        "  background-position: center;",
        "  background-repeat: no-repeat;",
        f"  background-image: url('https://res.cloudinary.com/{cloud_name}/image/upload/c_fill,f_webp,g_auto,q_auto:best,w_400/v{version}/{public_id}');",
        "}",
        "",
        "@media (min-width: 768px) {",
        "  .responsive-bg {",
        f"    background-image: url('https://res.cloudinary.com/{cloud_name}/image/upload/c_fill,f_webp,g_auto,q_auto:best,w_800/v{version}/{public_id}');",
        "  }",
        "}",
        "",
        "@media (min-width: 1200px) {",
        "  .responsive-bg {",
        f"    background-image: url('https://res.cloudinary.com/{cloud_name}/image/upload/c_fill,f_webp,g_auto,q_auto:best,w_1200/v{version}/{public_id}');",
        "  }",
        "}",
        "",
        "@media (min-width: 1800px) {",
        "  .responsive-bg {",
        f"    background-image: url('https://res.cloudinary.com/{cloud_name}/image/upload/c_fill,f_webp,g_auto,q_auto:best,w_1800/v{version}/{public_id}');",
        "  }",
        "}"
    ]
    
    return '\n'.join(css_parts)

def create_complete_html(image_html, result, variant):
    """Create a complete HTML document for testing"""
    
    return f"""<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Responsive Image Test - {variant}</title>
    <style>
        body {{
            font-family: system-ui, -apple-system, sans-serif;
            line-height: 1.6;
            margin: 0;
            padding: 2rem;
            background-color: #f5f5f5;
        }}
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            background: white;
            border-radius: 8px;
            padding: 2rem;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }}
        .image-container {{
            margin: 2rem 0;
        }}
        img, picture {{
            max-width: 100%;
            height: auto;
            display: block;
            margin: 0 auto;
            border-radius: 4px;
        }}
        .responsive-bg {{
            width: 100%;
            height: 400px;
            margin: 2rem 0;
            border-radius: 4px;
        }}
        .info {{
            background: #f8f9fa;
            padding: 1rem;
            border-radius: 4px;
            margin: 1rem 0;
            font-size: 0.9em;
        }}
        code {{
            background: #e9ecef;
            padding: 0.2em 0.4em;
            border-radius: 3px;
            font-family: 'SF Mono', Monaco, monospace;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Responsive Image Test: {variant.replace('_', ' ').title()}</h1>
        
        <div class="info">
            <strong>Original:</strong> {result['original_filename']}<br>
            <strong>Dimensions:</strong> {result['width']} × {result['height']}px<br>
            <strong>Size:</strong> {result['bytes']/1024:.1f} KB<br>
            <strong>Format:</strong> {result['format']}<br>
            <strong>Public ID:</strong> <code>{result['public_id']}</code>
        </div>
        
        <div class="image-container">
            {image_html}
        </div>
        
        <p><strong>Instructions:</strong> Resize your browser window to see the responsive behavior. 
        Open Developer Tools → Network tab to see which image sizes are loaded at different viewport widths.</p>
        
        <details>
            <summary>View HTML Source</summary>
            <pre><code>{image_html.replace('<', '&lt;').replace('>', '&gt;')}</code></pre>
        </details>
    </div>
</body>
</html>"""

if __name__ == "__main__":
    import sys
    if len(sys.argv) < 2:
        print("Usage: python3 upload_and_save.py <image_file_path> [alt_text] [aspect_ratio]")
        print("Example: python3 upload_and_save.py photo.jpg 'Beautiful landscape' '16:9'")
        sys.exit(1)
    
    image_path = sys.argv[1]
    alt_text = sys.argv[2] if len(sys.argv) > 2 else None
    aspect_ratio = sys.argv[3] if len(sys.argv) > 3 else None
    
    upload_image(image_path, alt_text=alt_text, aspect_ratio=aspect_ratio)