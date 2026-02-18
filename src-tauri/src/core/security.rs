pub struct SecurityPolicy;

impl SecurityPolicy {
    pub fn is_url_allowed(url: &str) -> bool {
        // Only allow http and https
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return false;
        }

        // Domain allowlist
        const ALLOWED_DOMAINS: &[&str] = &[
            "chat.openai.com",
            "chat.deepseek.com",
            "chatglm.cn", // z.ai / GLM
            "kimi.moonshot.cn",
            "github.com",
            "google.com", // For login redirects
        ];

        // Parse host... simplifying for now:
        // check if url contains one of the domains. proper parsing needed for production.
        for domain in ALLOWED_DOMAINS {
            if url.contains(domain) {
                return true;
            }
        }

        false
    }

    pub fn is_image_allowed(image: &str) -> bool {
        // Whitelist of allowed docker images
        const ALLOWED_IMAGES: &[&str] = &["python:3.9-alpine", "node:18-alpine"];
        ALLOWED_IMAGES.contains(&image)
    }
}
