# pxtwitter-mosaic
![](https://skillicons.dev/icons?i=rust,workers,wasm)

Rewrite of [pxTwitter-Mosaic](https://github.com/dangeredwolf/pxTwitter-Mosaic) in Rust for Cloudflare Workers.  
This has the same algorithm and functionality as the original project, with some exceptions:
- images are resized with `triangle` instead of `lanczos3`
- images will get resized by 50% if there's 4 of them and if they're wider or taller than 2000 pixels

# Notice
![](https://forthebadge.com/images/badges/contains-tasty-spaghetti-code.svg)

This is still very experimental and needs to be tested before using in production.  
I'm also somewhat new to Rust, so some code can definitely be improved.  
This is also pretty expensive to run, since at the moment it does not implement caching (soon™).
