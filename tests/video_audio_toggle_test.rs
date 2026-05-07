use rs_trafilatura::{extract_with_options, AudioData, Options, VideoData};

fn videos_contain_src(videos: &[VideoData], src: &str) -> bool {
    videos.iter().any(|v| v.src == src)
}

fn audio_contain_src(audio: &[AudioData], src: &str) -> bool {
    audio.iter().any(|a| a.src == src)
}

// ============================================================================
// VIDEO TOGGLE TESTS
// ============================================================================

#[test]
fn include_videos_true_collects_video_urls() {
    let html = r#"
        <html><body>
            <article>
                <p>Article with video content.</p>
                <video src="https://example.com/video1.mp4" controls></video>
                <p>More text here to ensure extraction succeeds with enough content.</p>
                <p>Additional paragraph for meeting minimum length requirements.</p>
            </article>
        </body></html>
    "#;

    let options = Options {
        include_videos: true,
        ..Options::default()
    };

    let result = extract_with_options(html, &options).expect("extraction failed");

    assert!(result.content_text.contains("Article with video"));
    assert!(!result.videos.is_empty());
    assert!(videos_contain_src(&result.videos, "https://example.com/video1.mp4"));
}

#[test]
fn include_videos_false_returns_empty_videos() {
    let html = r#"
        <html><body>
            <article>
                <p>Article with video content.</p>
                <video src="https://example.com/video1.mp4" controls></video>
                <p>More text here to ensure extraction succeeds with enough content.</p>
                <p>Additional paragraph for meeting minimum length requirements.</p>
            </article>
        </body></html>
    "#;

    let options = Options {
        include_videos: false,
        ..Options::default()
    };

    let result = extract_with_options(html, &options).expect("extraction failed");

    assert!(result.content_text.contains("Article with video"));
    assert!(result.videos.is_empty());
}

#[test]
fn default_options_excludes_videos() {
    let html = r#"
        <html><body>
            <article>
                <p>Content here.</p>
                <video src="https://example.com/video.mp4"></video>
                <p>More text here to ensure extraction succeeds with enough content.</p>
                <p>Additional paragraph for meeting minimum length requirements.</p>
            </article>
        </body></html>
    "#;

    let result = extract_with_options(html, &Options::default()).expect("extraction failed");

    assert!(result.videos.is_empty());
}

#[test]
fn videos_appear_in_html_output() {
    let html = r#"
        <html><body>
            <article>
                <p>Article with video.</p>
                <figure>
                    <video src="https://example.com/video.mp4" poster="https://example.com/poster.jpg" controls>
                        <source src="https://example.com/video.webm" type="video/webm">
                        <source src="https://example.com/video.mp4" type="video/mp4">
                        <track kind="subtitles" src="https://example.com/subtitles.vtt" srclang="en" label="English" default>
                    </video>
                    <figcaption>Video caption</figcaption>
                </figure>
                <p>More text here to ensure extraction succeeds with enough content.</p>
                <p>Additional paragraph for meeting minimum length requirements.</p>
            </article>
        </body></html>
    "#;

    let options = Options {
        include_videos: true,
        ..Options::default()
    };

    let result = extract_with_options(html, &options).expect("extraction failed");

    let content_html = result.content_html.expect("content_html should exist");

    assert!(content_html.contains("<video"), "HTML should contain <video>");
    assert!(content_html.contains("<source"), "HTML should contain <source>");
    assert!(content_html.contains("<track"), "HTML should contain <track>");
    assert!(content_html.contains("<figcaption"), "HTML should contain <figcaption>");
    assert!(content_html.contains("src=\"https://example.com/video.mp4\""), "HTML should contain video src");
    assert!(content_html.contains("poster=\"https://example.com/poster.jpg\""), "HTML should contain poster");
    assert!(content_html.contains("kind=\"subtitles\""), "HTML should contain track kind");
    assert!(content_html.contains("srclang=\"en\""), "HTML should contain track srclang");
}

#[test]
fn videos_appear_in_markdown_output() {
    let html = r#"
        <html><body>
            <article>
                <p>Article with video.</p>
                <video src="https://example.com/video.mp4" controls></video>
                <p>More text here to ensure extraction succeeds with enough content.</p>
                <p>Additional paragraph for meeting minimum length requirements.</p>
            </article>
        </body></html>
    "#;

    let options = Options {
        include_videos: true,
        output_markdown: true,
        ..Options::default()
    };

    let result = extract_with_options(html, &options).expect("extraction failed");

    // Markdown doesn't have native video syntax, so quick_html2md strips the tag.
    // But the text content should still be preserved.
    let content_md = result.content_markdown.expect("content_markdown should exist");
    assert!(content_md.contains("Article with video"), "Markdown should contain text: {content_md}");
    assert!(result.videos.len() == 1, "Video should be extracted");
}

#[test]
fn video_in_figure_with_figcaption() {
    let html = r#"
        <html><body>
            <article>
                <p>Article text content here.</p>
                <figure>
                    <video src="https://example.com/featured.mp4" poster="https://example.com/thumb.jpg"></video>
                    <figcaption>This is the featured video.</figcaption>
                </figure>
                <p>More text here to ensure extraction succeeds with enough content.</p>
                <p>Additional paragraph for meeting minimum length requirements.</p>
            </article>
        </body></html>
    "#;

    let options = Options {
        include_videos: true,
        ..Options::default()
    };

    let result = extract_with_options(html, &options).expect("extraction failed");

    assert_eq!(result.videos.len(), 1);
    assert_eq!(result.videos[0].src, "https://example.com/featured.mp4");
    assert_eq!(result.videos[0].poster, Some("https://example.com/thumb.jpg".to_string()));
    assert_eq!(result.videos[0].caption, Some("This is the featured video.".to_string()));
}

#[test]
fn video_with_multiple_sources() {
    let html = r#"
        <html><body>
            <article>
                <p>Article with multi-source video.</p>
                <video controls>
                    <source src="https://example.com/video.webm" type="video/webm">
                    <source src="https://example.com/video.mp4" type="video/mp4">
                </video>
                <p>More text here to ensure extraction succeeds with enough content.</p>
                <p>Additional paragraph for meeting minimum length requirements.</p>
            </article>
        </body></html>
    "#;

    let options = Options {
        include_videos: true,
        ..Options::default()
    };

    let result = extract_with_options(html, &options).expect("extraction failed");

    assert_eq!(result.videos.len(), 1);
    // Should use first source
    assert_eq!(result.videos[0].src, "https://example.com/video.webm");
    assert_eq!(result.videos[0].filename, "video.webm");
}

#[test]
fn video_with_track_subtitles() {
    let html = r#"
        <html><body>
            <article>
                <p>Article with subtitled video.</p>
                <video src="https://example.com/video.mp4" controls>
                    <track kind="subtitles" src="https://example.com/en.vtt" srclang="en" label="English" default>
                    <track kind="subtitles" src="https://example.com/es.vtt" srclang="es" label="Spanish">
                </video>
                <p>More text here to ensure extraction succeeds with enough content.</p>
                <p>Additional paragraph for meeting minimum length requirements.</p>
            </article>
        </body></html>
    "#;

    let options = Options {
        include_videos: true,
        ..Options::default()
    };

    let result = extract_with_options(html, &options).expect("extraction failed");

    assert_eq!(result.videos.len(), 1);
    assert_eq!(result.videos[0].src, "https://example.com/video.mp4");

    let content_html = result.content_html.expect("content_html should exist");
    assert!(content_html.contains("kind=\"subtitles\""));
    assert!(content_html.contains("srclang=\"en\""));
    assert!(content_html.contains("srclang=\"es\""));
}

#[test]
fn video_deduplicates_urls() {
    let html = r#"
        <html><body>
            <article>
                <p>Article with duplicate videos.</p>
                <video src="https://example.com/same.mp4"></video>
                <video src="https://example.com/same.mp4"></video>
                <p>More text here to ensure extraction succeeds with enough content.</p>
                <p>Additional paragraph for meeting minimum length requirements.</p>
            </article>
        </body></html>
    "#;

    let options = Options {
        include_videos: true,
        ..Options::default()
    };

    let result = extract_with_options(html, &options).expect("extraction failed");

    assert_eq!(result.videos.len(), 1);
}

#[test]
fn video_toggle_doesnt_affect_text_content() {
    let html = r#"
        <html><body>
            <article>
                <p>Article with video content that should be extracted identically.</p>
                <video src="https://example.com/video.mp4"></video>
                <p>More text here to ensure extraction succeeds with enough content.</p>
                <p>Additional paragraph for meeting minimum length requirements.</p>
            </article>
        </body></html>
    "#;

    let with_videos = extract_with_options(html, &Options {
        include_videos: true,
        ..Options::default()
    }).expect("extraction failed");

    let without_videos = extract_with_options(html, &Options {
        include_videos: false,
        ..Options::default()
    }).expect("extraction failed");

    assert_eq!(with_videos.content_text, without_videos.content_text);
}

#[test]
fn video_preserves_poster_attribute() {
    let html = r#"
        <html><body>
            <article>
                <p>Article with poster video.</p>
                <video src="https://example.com/video.mp4" poster="https://example.com/poster.jpg"></video>
                <p>More text here to ensure extraction succeeds with enough content.</p>
                <p>Additional paragraph for meeting minimum length requirements.</p>
            </article>
        </body></html>
    "#;

    let options = Options {
        include_videos: true,
        ..Options::default()
    };

    let result = extract_with_options(html, &options).expect("extraction failed");

    assert_eq!(result.videos.len(), 1);
    assert_eq!(result.videos[0].poster, Some("https://example.com/poster.jpg".to_string()));
}

#[test]
fn fallback_preserves_video_html() {
    let html = r#"
        <html><body>
            <article>
                <p>Article with video that should be preserved in fallback.</p>
                <video src="https://example.com/video.mp4" controls></video>
                <p>More text here to ensure extraction succeeds with enough content.</p>
                <p>Additional paragraph for meeting minimum length requirements.</p>
            </article>
        </body></html>
    "#;

    let options = Options {
        include_videos: true,
        ..Options::default()
    };

    let result = extract_with_options(html, &options).expect("extraction failed");

    let content_html = result.content_html.expect("content_html should exist");
    assert!(content_html.contains("<video"), "HTML should preserve video tag");
    assert!(content_html.contains("https://example.com/video.mp4"), "HTML should preserve video src");
}

// ============================================================================
// AUDIO TOGGLE TESTS
// ============================================================================

#[test]
fn include_audio_true_collects_audio_urls() {
    let html = r#"
        <html><body>
            <article>
                <p>Article with audio content.</p>
                <audio src="https://example.com/audio1.mp3" controls></audio>
                <p>More text here to ensure extraction succeeds with enough content.</p>
                <p>Additional paragraph for meeting minimum length requirements.</p>
            </article>
        </body></html>
    "#;

    let options = Options {
        include_audio: true,
        ..Options::default()
    };

    let result = extract_with_options(html, &options).expect("extraction failed");

    assert!(result.content_text.contains("Article with audio"));
    assert!(!result.audio.is_empty());
    assert!(audio_contain_src(&result.audio, "https://example.com/audio1.mp3"));
}

#[test]
fn include_audio_false_returns_empty_audio() {
    let html = r#"
        <html><body>
            <article>
                <p>Article with audio content.</p>
                <audio src="https://example.com/audio1.mp3" controls></audio>
                <p>More text here to ensure extraction succeeds with enough content.</p>
                <p>Additional paragraph for meeting minimum length requirements.</p>
            </article>
        </body></html>
    "#;

    let options = Options {
        include_audio: false,
        ..Options::default()
    };

    let result = extract_with_options(html, &options).expect("extraction failed");

    assert!(result.content_text.contains("Article with audio"));
    assert!(result.audio.is_empty());
}

#[test]
fn default_options_excludes_audio() {
    let html = r#"
        <html><body>
            <article>
                <p>Content here.</p>
                <audio src="https://example.com/audio.mp3"></audio>
                <p>More text here to ensure extraction succeeds with enough content.</p>
                <p>Additional paragraph for meeting minimum length requirements.</p>
            </article>
        </body></html>
    "#;

    let result = extract_with_options(html, &Options::default()).expect("extraction failed");

    assert!(result.audio.is_empty());
}

#[test]
fn audio_appear_in_html_output() {
    let html = r#"
        <html><body>
            <article>
                <p>Article with audio.</p>
                <figure>
                    <audio src="https://example.com/audio.mp3" controls>
                        <source src="https://example.com/audio.ogg" type="audio/ogg">
                        <source src="https://example.com/audio.mp3" type="audio/mpeg">
                    </audio>
                    <figcaption>Audio caption</figcaption>
                </figure>
                <p>More text here to ensure extraction succeeds with enough content.</p>
                <p>Additional paragraph for meeting minimum length requirements.</p>
            </article>
        </body></html>
    "#;

    let options = Options {
        include_audio: true,
        ..Options::default()
    };

    let result = extract_with_options(html, &options).expect("extraction failed");

    let content_html = result.content_html.expect("content_html should exist");

    assert!(content_html.contains("<audio"), "HTML should contain <audio>");
    assert!(content_html.contains("<source"), "HTML should contain <source>");
    assert!(content_html.contains("<figcaption"), "HTML should contain <figcaption>");
    assert!(content_html.contains("src=\"https://example.com/audio.mp3\""), "HTML should contain audio src");
}

#[test]
fn audio_appear_in_markdown_output() {
    let html = r#"
        <html><body>
            <article>
                <p>Article with audio.</p>
                <audio src="https://example.com/audio.mp3" controls></audio>
                <p>More text here to ensure extraction succeeds with enough content.</p>
                <p>Additional paragraph for meeting minimum length requirements.</p>
            </article>
        </body></html>
    "#;

    let options = Options {
        include_audio: true,
        output_markdown: true,
        ..Options::default()
    };

    let result = extract_with_options(html, &options).expect("extraction failed");

    // Markdown doesn't have native audio syntax, so quick_html2md strips the tag.
    // But the text content should still be preserved.
    let content_md = result.content_markdown.expect("content_markdown should exist");
    assert!(content_md.contains("Article with audio"), "Markdown should contain text: {content_md}");
    assert!(result.audio.len() == 1, "Audio should be extracted");
}

#[test]
fn audio_in_figure_with_figcaption() {
    let html = r#"
        <html><body>
            <article>
                <p>Article text content here.</p>
                <figure>
                    <audio src="https://example.com/podcast.mp3"></audio>
                    <figcaption>Episode 42: The Future of Rust</figcaption>
                </figure>
                <p>More text here to ensure extraction succeeds with enough content.</p>
                <p>Additional paragraph for meeting minimum length requirements.</p>
            </article>
        </body></html>
    "#;

    let options = Options {
        include_audio: true,
        ..Options::default()
    };

    let result = extract_with_options(html, &options).expect("extraction failed");

    assert_eq!(result.audio.len(), 1);
    assert_eq!(result.audio[0].src, "https://example.com/podcast.mp3");
    assert_eq!(result.audio[0].caption, Some("Episode 42: The Future of Rust".to_string()));
}

#[test]
fn audio_with_multiple_sources() {
    let html = r#"
        <html><body>
            <article>
                <p>Article with multi-source audio.</p>
                <audio controls>
                    <source src="https://example.com/audio.ogg" type="audio/ogg">
                    <source src="https://example.com/audio.mp3" type="audio/mpeg">
                </audio>
                <p>More text here to ensure extraction succeeds with enough content.</p>
                <p>Additional paragraph for meeting minimum length requirements.</p>
            </article>
        </body></html>
    "#;

    let options = Options {
        include_audio: true,
        ..Options::default()
    };

    let result = extract_with_options(html, &options).expect("extraction failed");

    assert_eq!(result.audio.len(), 1);
    assert_eq!(result.audio[0].src, "https://example.com/audio.ogg");
    assert_eq!(result.audio[0].filename, "audio.ogg");
}

#[test]
fn audio_deduplicates_urls() {
    let html = r#"
        <html><body>
            <article>
                <p>Article with duplicate audio.</p>
                <audio src="https://example.com/same.mp3"></audio>
                <audio src="https://example.com/same.mp3"></audio>
                <p>More text here to ensure extraction succeeds with enough content.</p>
                <p>Additional paragraph for meeting minimum length requirements.</p>
            </article>
        </body></html>
    "#;

    let options = Options {
        include_audio: true,
        ..Options::default()
    };

    let result = extract_with_options(html, &options).expect("extraction failed");

    assert_eq!(result.audio.len(), 1);
}

#[test]
fn audio_toggle_doesnt_affect_text_content() {
    let html = r#"
        <html><body>
            <article>
                <p>Article with audio content that should be extracted identically.</p>
                <audio src="https://example.com/audio.mp3"></audio>
                <p>More text here to ensure extraction succeeds with enough content.</p>
                <p>Additional paragraph for meeting minimum length requirements.</p>
            </article>
        </body></html>
    "#;

    let with_audio = extract_with_options(html, &Options {
        include_audio: true,
        ..Options::default()
    }).expect("extraction failed");

    let without_audio = extract_with_options(html, &Options {
        include_audio: false,
        ..Options::default()
    }).expect("extraction failed");

    assert_eq!(with_audio.content_text, without_audio.content_text);
}

#[test]
fn fallback_preserves_audio_html() {
    let html = r#"
        <html><body>
            <article>
                <p>Article with audio that should be preserved in fallback.</p>
                <audio src="https://example.com/audio.mp3" controls></audio>
                <p>More text here to ensure extraction succeeds with enough content.</p>
                <p>Additional paragraph for meeting minimum length requirements.</p>
            </article>
        </body></html>
    "#;

    let options = Options {
        include_audio: true,
        ..Options::default()
    };

    let result = extract_with_options(html, &options).expect("extraction failed");

    let content_html = result.content_html.expect("content_html should exist");
    assert!(content_html.contains("<audio"), "HTML should preserve audio tag");
    assert!(content_html.contains("https://example.com/audio.mp3"), "HTML should preserve audio src");
}
