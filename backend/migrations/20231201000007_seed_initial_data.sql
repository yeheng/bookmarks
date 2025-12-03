-- Seed initial data for the bookmarks application

-- Insert two test users
INSERT INTO users (
    id,
    username,
    email,
    password_hash,
    avatar_url,
    is_active,
    email_verified,
    last_login_at
) VALUES
(
    1,
    'Heng Ye',
    'hengheng8848@gmail.com',
    '$2b$12$D65CTEexd9I6ElfjCwe/WeD5RtyI6UHNpfFos/rD9FY/eknl1xdnm', -- password: password123
    'https://ui-avatars.com/api/?name=John+Doe&background=3b82f6&color=fff',
    1,
    1,
    datetime('now', '-2 days')
),
(
    2,
    'jane_smith',
    'jane.smith@example.com',
    '$2b$12$D65CTEexd9I6ElfjCwe/WeD5RtyI6UHNpfFos/rD9FY/eknl1xdnm', -- password: password123
    'https://ui-avatars.com/api/?name=Jane+Smith&background=ec4899&color=fff',
    1,
    1,
    datetime('now', '-1 day')
);

-- Initialize bookmark stats for the users
INSERT INTO bookmark_stats (user_id, total_bookmarks, favorite_count, archived_count, unread_count, total_visits) VALUES
(1, 0, 0, 0, 0, 0),
(2, 0, 0, 0, 0, 0);

-- Insert collections for John Doe
INSERT INTO collections (
    id,
    user_id,
    name,
    description,
    color,
    icon,
    sort_order,
    is_default,
    is_public
) VALUES
(
    1,
    1,
    'Development',
    'Programming resources, tools, and documentation',
    '#3b82f6',
    'code',
    1,
    1,
    0
),
(
    2,
    1,
    'Design',
    'UI/UX design resources and inspiration',
    '#ec4899',
    'palette',
    2,
    0,
    0
),
(
    3,
    1,
    'Learning',
    'Educational content and tutorials',
    '#10b981',
    'graduation-cap',
    3,
    0,
    0
);

-- Insert collections for Jane Smith
INSERT INTO collections (
    id,
    user_id,
    name,
    description,
    color,
    icon,
    sort_order,
    is_default,
    is_public
) VALUES
(
    4,
    2,
    'Research',
    'Academic papers and research materials',
    '#8b5cf6',
    'search',
    1,
    1,
    0
),
(
    5,
    2,
    'News',
    'News articles and current events',
    '#f59e0b',
    'newspaper',
    2,
    0,
    0
);

-- Insert tags for John Doe
INSERT INTO tags (
    id,
    user_id,
    name,
    color,
    description,
    usage_count
) VALUES
(
    1,
    1,
    'rust',
    '#ce422b',
    'Rust programming language resources',
    0
),
(
    2,
    1,
    'web-development',
    '#3b82f6',
    'Web development articles and tutorials',
    0
),
(
    3,
    1,
    'database',
    '#10b981',
    'Database design and optimization',
    0
),
(
    4,
    1,
    'tutorial',
    '#f59e0b',
    'Learning tutorials and guides',
    0
);

-- Insert tags for Jane Smith
INSERT INTO tags (
    id,
    user_id,
    name,
    color,
    description,
    usage_count
) VALUES
(
    5,
    2,
    'machine-learning',
    '#8b5cf6',
    'Machine learning and AI resources',
    0
),
(
    6,
    2,
    'data-science',
    '#06b6d4',
    'Data science articles and research',
    0
),
(
    7,
    2,
    'python',
    '#3776ab',
    'Python programming resources',
    0
);

-- Insert sample bookmarks for John Doe
INSERT INTO bookmarks (
    id,
    user_id,
    collection_id,
    title,
    url,
    description,
    favicon_url,
    is_favorite,
    is_archived,
    is_private,
    is_read,
    visit_count,
    last_visited,
    reading_time,
    difficulty_level,
    metadata
) VALUES
(
    1,
    1,
    1,
    'Rust Programming Language',
    'https://www.rust-lang.org/',
    'A systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety.',
    'https://www.rust-lang.org/favicon.ico',
    1,
    0,
    0,
    1,
    15,
    datetime('now', '-1 day'),
    8,
    2,
    '{"language": "en", "type": "documentation"}'
),
(
    2,
    1,
    1,
    'MDN Web Docs',
    'https://developer.mozilla.org/',
    'Resources for developers, by developers. Comprehensive documentation for web technologies.',
    'https://developer.mozilla.org/favicon.ico',
    1,
    0,
    0,
    1,
    23,
    datetime('now', '-3 hours'),
    12,
    1,
    '{"language": "en", "type": "documentation"}'
),
(
    3,
    1,
    3,
    'Rust Book',
    'https://doc.rust-lang.org/book/',
    'The Rust Programming Language book is the official resource for learning Rust.',
    'https://doc.rust-lang.org/favicon.ico',
    1,
    0,
    0,
    0,
    8,
    datetime('now', '-2 days'),
    25,
    2,
    '{"language": "en", "type": "tutorial"}'
),
(
    4,
    1,
    2,
    'Dribbble',
    'https://dribbble.com/',
    'Discover the world''s top designers & creatives. Dribbble is the leading destination to find & showcase creative work.',
    'https://cdn.dribbble.com/assets/favicon-63b2904a073c89b52b19aa05c6a21e32.ico',
    0,
    0,
    0,
    1,
    5,
    datetime('now', '-5 hours'),
    6,
    1,
    '{"language": "en", "type": "inspiration"}'
);

-- Insert sample bookmarks for Jane Smith
INSERT INTO bookmarks (
    id,
    user_id,
    collection_id,
    title,
    url,
    description,
    favicon_url,
    is_favorite,
    is_archived,
    is_private,
    is_read,
    visit_count,
    last_visited,
    reading_time,
    difficulty_level,
    metadata
) VALUES
(
    5,
    2,
    4,
    'arXiv.org',
    'https://arxiv.org/',
    'arXiv is a free distribution service and an open-access archive for nearly 2.4 million scholarly articles.',
    'https://arxiv.org/favicon.ico',
    1,
    0,
    0,
    1,
    12,
    datetime('now', '-4 hours'),
    15,
    4,
    '{"language": "en", "type": "research"}'
),
(
    6,
    2,
    4,
    'Papers With Code',
    'https://paperswithcode.com/',
    'Latest machine learning papers with code, trending ML repositories and datasets.',
    'https://paperswithcode.com/favicon.ico',
    1,
    0,
    0,
    0,
    7,
    datetime('now', '-1 day'),
    10,
    3,
    '{"language": "en", "type": "research"}'
),
(
    7,
    2,
    5,
    'TechCrunch',
    'https://techcrunch.com/',
    'Latest technology news and updates on startups, gadgets, and innovation.',
    'https://techcrunch.com/favicon.ico',
    0,
    0,
    0,
    1,
    3,
    datetime('now', '-30 minutes'),
    5,
    1,
    '{"language": "en", "type": "news"}'
);

-- Insert bookmark-tag relationships for John Doe's bookmarks
INSERT INTO bookmark_tags (bookmark_id, tag_id) VALUES
(1, 1), -- Rust Programming Language -> rust
(1, 3), -- Rust Programming Language -> database
(2, 2), -- MDN Web Docs -> web-development
(3, 1), -- Rust Book -> rust
(3, 4), -- Rust Book -> tutorial
(4, 2); -- Dribbble -> web-development

-- Insert bookmark-tag relationships for Jane Smith's bookmarks
INSERT INTO bookmark_tags (bookmark_id, tag_id) VALUES
(5, 5), -- arXiv.org -> machine-learning
(5, 6), -- arXiv.org -> data-science
(6, 5), -- Papers With Code -> machine-learning
(6, 6), -- Papers With Code -> data-science
(6, 7); -- Papers With Code -> python

-- Update tag usage counts
UPDATE tags SET usage_count = (
    SELECT COUNT(*)
    FROM bookmark_tags bt
    JOIN bookmarks b ON bt.bookmark_id = b.id
    WHERE bt.tag_id = tags.id AND b.is_archived = 0
);

-- Update collection bookmark counts
UPDATE collections SET bookmark_count = (
    SELECT COUNT(*)
    FROM bookmarks
    WHERE collection_id = collections.id AND is_archived = 0
);
