-- Add more comprehensive test data for better testing experience

-- First, ensure the initial seed data exists by inserting it if not present
INSERT OR IGNORE INTO users (
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
    CAST(strftime('%s', 'now', '-2 days') AS INTEGER)
),
(
    2,
    'jane_smith',
    'jane.smith@example.com',
    '$2b$12$D65CTEexd9I6ElfjCwe/WeD5RtyI6UHNpfFos/rD9FY/eknl1xdnm', -- password: password123
    'https://ui-avatars.com/api/?name=Jane+Smith&background=ec4899&color=fff',
    1,
    1,
    CAST(strftime('%s', 'now', '-1 day') AS INTEGER)
);

-- Insert base collections for John Doe (if not already present)
INSERT OR IGNORE INTO collections (
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

-- Insert base collections for Jane Smith (if not already present)
INSERT OR IGNORE INTO collections (
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

-- Insert base tags for John Doe (if not already present)
INSERT OR IGNORE INTO tags (
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

-- Insert base tags for Jane Smith (if not already present)
INSERT OR IGNORE INTO tags (
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

-- Insert additional collections for John Doe (user_id = 1)
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
    100,
    1,
    'DevOps',
    'DevOps tools, CI/CD, and infrastructure resources',
    '#ef4444',
    'server',
    4,
    0,
    0
),
(
    101,
    1,
    'Mobile Development',
    'iOS and Android development resources',
    '#a855f7',
    'smartphone',
    5,
    0,
    0
),
(
    102,
    1,
    'Security',
    'Cybersecurity and application security resources',
    '#059669',
    'shield',
    6,
    0,
    0
),
(
    103,
    1,
    'Productivity',
    'Tools and techniques for better productivity',
    '#f97316',
    'zap',
    7,
    0,
    0
);

-- Insert additional collections for Jane Smith (user_id = 2)
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
    104,
    2,
    'Healthcare',
    'Healthcare technology and medical research',
    '#dc2626',
    'heart',
    3,
    0,
    0
),
(
    105,
    2,
    'Finance',
    'Financial technology and investment resources',
    '#16a34a',
    'dollar-sign',
    4,
    0,
    0
),
(
    106,
    2,
    'Climate',
    'Climate change and environmental research',
    '#0891b2',
    'leaf',
    5,
    0,
    0
);

-- Insert base bookmarks for John Doe (if not already present)
INSERT OR IGNORE INTO bookmarks (
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
    CAST(strftime('%s', 'now', '-1 day') AS INTEGER),
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
    CAST(strftime('%s', 'now', '-3 hours') AS INTEGER),
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
    CAST(strftime('%s', 'now', '-2 days') AS INTEGER),
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
    CAST(strftime('%s', 'now', '-5 hours') AS INTEGER),
    6,
    1,
    '{"language": "en", "type": "inspiration"}'
);

-- Insert base bookmarks for Jane Smith (if not already present)
INSERT OR IGNORE INTO bookmarks (
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
    CAST(strftime('%s', 'now', '-4 hours') AS INTEGER),
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
    CAST(strftime('%s', 'now', '-1 day') AS INTEGER),
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
    CAST(strftime('%s', 'now', '-30 minutes') AS INTEGER),
    5,
    1,
    '{"language": "en", "type": "news"}'
);

-- Insert base bookmark-tag relationships for John Doe's bookmarks (if not already present)
INSERT OR IGNORE INTO bookmark_tags (bookmark_id, tag_id) VALUES
(1, 1), -- Rust Programming Language -> rust
(1, 3), -- Rust Programming Language -> database
(2, 2), -- MDN Web Docs -> web-development
(3, 1), -- Rust Book -> rust
(3, 4), -- Rust Book -> tutorial
(4, 2); -- Dribbble -> web-development

-- Insert base bookmark-tag relationships for Jane Smith's bookmarks (if not already present)
INSERT OR IGNORE INTO bookmark_tags (bookmark_id, tag_id) VALUES
(5, 5), -- arXiv.org -> machine-learning
(5, 6), -- arXiv.org -> data-science
(6, 5), -- Papers With Code -> machine-learning
(6, 6), -- Papers With Code -> data-science
(6, 7); -- Papers With Code -> python

-- Insert additional tags for John Doe (user_id = 1)
INSERT INTO tags (
    id,
    user_id,
    name,
    color,
    description,
    usage_count
) VALUES
(
    100,
    1,
    'docker',
    '#2496ed',
    'Docker containerization resources',
    0
),
(
    101,
    1,
    'kubernetes',
    '#326ce5',
    'Kubernetes orchestration tutorials',
    0
),
(
    102,
    1,
    'react',
    '#61dafb',
    'React.js framework resources',
    0
),
(
    103,
    1,
    'vue',
    '#4fc08d',
    'Vue.js framework resources',
    0
),
(
    104,
    1,
    'typescript',
    '#3178c6',
    'TypeScript programming language',
    0
),
(
    105,
    1,
    'api',
    '#ff6b6b',
    'API design and development',
    0
),
(
    106,
    1,
    'testing',
    '#feca57',
    'Software testing and QA',
    0
),
(
    107,
    1,
    'performance',
    '#ff9ff3',
    'Performance optimization',
    0
),
(
    108,
    1,
    'security',
    '#ee5a6f',
    'Application security best practices',
    0
),
(
    109,
    1,
    'mobile',
    '#00d2d3',
    'Mobile app development',
    0
),
(
    110,
    1,
    'ios',
    '#000000',
    'iOS development resources',
    0
),
(
    111,
    1,
    'android',
    '#3ddc84',
    'Android development resources',
    0
),
(
    112,
    1,
    'productivity',
    '#f368e0',
    'Productivity tools and techniques',
    0
),
(
    113,
    1,
    'automation',
    '#00b894',
    'Automation tools and scripts',
    0
);

-- Insert additional tags for Jane Smith (user_id = 2)
INSERT INTO tags (
    id,
    user_id,
    name,
    color,
    description,
    usage_count
) VALUES
(
    114,
    2,
    'deep-learning',
    '#ff6384',
    'Deep learning frameworks and tutorials',
    0
),
(
    115,
    2,
    'nlp',
    '#36a2eb',
    'Natural Language Processing resources',
    0
),
(
    116,
    2,
    'tensorflow',
    '#ff6f00',
    'TensorFlow framework resources',
    0
),
(
    117,
    2,
    'pytorch',
    '#ee4c2c',
    'PyTorch framework resources',
    0
),
(
    118,
    2,
    'healthcare',
    '#4caf50',
    'Healthcare technology and research',
    0
),
(
    119,
    2,
    'fintech',
    '#2196f3',
    'Financial technology innovations',
    0
),
(
    120,
    2,
    'climate',
    '#00bcd4',
    'Climate change research and data',
    0
),
(
    121,
    2,
    'statistics',
    '#9c27b0',
    'Statistical analysis and methods',
    0
);

-- Insert many more bookmarks for John Doe
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
-- DevOps bookmarks
(
    100,
    1,
    100,
    'Docker Documentation',
    'https://docs.docker.com/',
    'Official Docker documentation covering containerization, Dockerfiles, and orchestration.',
    'https://docs.docker.com/favicon.ico',
    1,
    0,
    0,
    1,
    25,
    CAST(strftime('%s', 'now', '-6 hours') AS INTEGER),
    20,
    2,
    '{"language": "en", "type": "documentation"}'
),
(
    101,
    1,
    100,
    'Kubernetes Documentation',
    'https://kubernetes.io/docs/',
    'Production-grade container orchestration system documentation.',
    'https://kubernetes.io/favicon.ico',
    1,
    0,
    0,
    0,
    18,
    CAST(strftime('%s', 'now', '-2 days') AS INTEGER),
    35,
    3,
    '{"language": "en", "type": "documentation"}'
),
(
    102,
    1,
    100,
    'GitHub Actions',
    'https://docs.github.com/en/actions',
    'Automate your workflow from idea to production with GitHub Actions.',
    'https://github.com/favicon.ico',
    0,
    0,
    0,
    1,
    12,
    CAST(strftime('%s', 'now', '-12 hours') AS INTEGER),
    15,
    2,
    '{"language": "en", "type": "documentation"}'
),
(
    103,
    1,
    100,
    'Terraform by HashiCorp',
    'https://www.terraform.io/',
    'Infrastructure as Code tool to safely and predictably create, change, and improve infrastructure.',
    'https://www.terraform.io/favicon.ico',
    0,
    0,
    0,
    0,
    8,
    CAST(strftime('%s', 'now', '-3 days') AS INTEGER),
    12,
    2,
    '{"language": "en", "type": "documentation"}'
),
-- Mobile Development bookmarks
(
    104,
    1,
    101,
    'React Native Documentation',
    'https://reactnative.dev/docs/getting-started',
    'Build native mobile apps using React and JavaScript.',
    'https://reactnative.dev/favicon.ico',
    1,
    0,
    0,
    1,
    20,
    CAST(strftime('%s', 'now', '-8 hours') AS INTEGER),
    25,
    2,
    '{"language": "en", "type": "documentation"}'
),
(
    105,
    1,
    101,
    'Flutter Documentation',
    'https://flutter.dev/docs',
    'Google''s UI toolkit for building beautiful, natively compiled applications.',
    'https://flutter.dev/favicon.ico',
    0,
    0,
    0,
    0,
    15,
    CAST(strftime('%s', 'now', '-1 day') AS INTEGER),
    22,
    2,
    '{"language": "en", "type": "documentation"}'
),
(
    106,
    1,
    101,
    'iOS Developer Documentation',
    'https://developer.apple.com/documentation/',
    'Comprehensive documentation for iOS, macOS, and Apple platform development.',
    'https://developer.apple.com/favicon.ico',
    0,
    0,
    0,
    1,
    10,
    CAST(strftime('%s', 'now', '-4 hours') AS INTEGER),
    30,
    3,
    '{"language": "en", "type": "documentation"}'
),
(
    107,
    1,
    101,
    'Android Developers',
    'https://developer.android.com/',
    'Official Android development documentation and resources.',
    'https://developer.android.com/favicon.ico',
    0,
    0,
    0,
    0,
    14,
    CAST(strftime('%s', 'now', '-2 days') AS INTEGER),
    28,
    3,
    '{"language": "en", "type": "documentation"}'
),
-- Security bookmarks
(
    108,
    1,
    102,
    'OWASP Top Ten',
    'https://owasp.org/www-project-top-ten/',
    'The OWASP Top Ten is a standard awareness document for developers and web application security.',
    'https://owasp.org/favicon.ico',
    1,
    0,
    0,
    1,
    16,
    CAST(strftime('%s', 'now', '-5 hours') AS INTEGER),
    18,
    2,
    '{"language": "en", "type": "documentation"}'
),
(
    109,
    1,
    102,
    'Crash Course Computer Science',
    'https://www.youtube.com/playlist?list=PL8dPuuaLjXtNlUrzyH5r6jN9ulIgZBpdo',
    'Comprehensive computer science playlist covering security, algorithms, and more.',
    'https://www.youtube.com/favicon.ico',
    0,
    0,
    0,
    1,
    22,
    CAST(strftime('%s', 'now', '-1 day') AS INTEGER),
    45,
    1,
    '{"language": "en", "type": "video"}'
),
-- Productivity bookmarks
(
    110,
    1,
    103,
    'Notion',
    'https://www.notion.so/',
    'All-in-one workspace for note-taking, project management, and collaboration.',
    'https://www.notion.so/favicon.ico',
    1,
    0,
    0,
    1,
    30,
    CAST(strftime('%s', 'now', '-2 hours') AS INTEGER),
    10,
    1,
    '{"language": "en", "type": "tool"}'
),
(
    111,
    1,
    103,
    'Obsidian',
    'https://obsidian.md/',
    'A powerful knowledge base that works on local Markdown files.',
    'https://obsidian.md/favicon.ico',
    0,
    0,
    0,
    0,
    18,
    CAST(strftime('%s', 'now', '-3 days') AS INTEGER),
    8,
    1,
    '{"language": "en", "type": "tool"}'
),
-- More development bookmarks
(
    112,
    1,
    1,
    'Stack Overflow',
    'https://stackoverflow.com/',
    'Where developers learn, share, & build careers.',
    'https://stackoverflow.com/favicon.ico',
    1,
    0,
    0,
    1,
    45,
    CAST(strftime('%s', 'now', '-1 hour') AS INTEGER),
    15,
    1,
    '{"language": "en", "type": "community"}'
),
(
    113,
    1,
    1,
    'GitHub',
    'https://github.com/',
    'Where the world builds software.',
    'https://github.com/favicon.ico',
    1,
    0,
    0,
    1,
    60,
    CAST(strftime('%s', 'now', '-30 minutes') AS INTEGER),
    20,
    1,
    '{"language": "en", "type": "tool"}'
),
(
    114,
    1,
    1,
    'Vue.js Guide',
    'https://vuejs.org/guide/introduction.html',
    'The official guide for Vue.js, the progressive JavaScript framework.',
    'https://vuejs.org/favicon.ico',
    0,
    0,
    0,
    0,
    12,
    CAST(strftime('%s', 'now', '-6 hours') AS INTEGER),
    18,
    2,
    '{"language": "en", "type": "documentation"}'
),
(
    115,
    1,
    1,
    'TypeScript Handbook',
    'https://www.typescriptlang.org/docs/',
    'The TypeScript Handbook is a comprehensive guide to the TypeScript language.',
    'https://www.typescriptlang.org/favicon.ico',
    0,
    0,
    0,
    1,
    25,
    CAST(strftime('%s', 'now', '-4 hours') AS INTEGER),
    30,
    2,
    '{"language": "en", "type": "documentation"}'
);

-- Insert many more bookmarks for Jane Smith
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
-- Research bookmarks
(
    200,
    2,
    4,
    'Nature',
    'https://www.nature.com/',
    'International weekly journal of science.',
    'https://www.nature.com/favicon.ico',
    1,
    0,
    0,
    1,
    28,
    CAST(strftime('%s', 'now', '-3 hours') AS INTEGER),
    25,
    4,
    '{"language": "en", "type": "journal"}'
),
(
    201,
    2,
    4,
    'Science Magazine',
    'https://www.science.org/',
    'Leading journal of original scientific research, global news, and commentary.',
    'https://www.science.org/favicon.ico',
    0,
    0,
    0,
    0,
    15,
    CAST(strftime('%s', 'now', '-1 day') AS INTEGER),
    22,
    4,
    '{"language": "en", "type": "journal"}'
),
(
    202,
    2,
    4,
    'Google Scholar',
    'https://scholar.google.com/',
    'Broadly search for scholarly literature.',
    'https://scholar.google.com/favicon.ico',
    1,
    0,
    0,
    1,
    35,
    CAST(strftime('%s', 'now', '-2 hours') AS INTEGER),
    12,
    1,
    '{"language": "en", "type": "search"}'
),
-- Healthcare bookmarks
(
    203,
    2,
    104,
    'PubMed',
    'https://pubmed.ncbi.nlm.nih.gov/',
    'PubMed comprises more than 32 million citations for biomedical literature.',
    'https://pubmed.ncbi.nlm.nih.gov/favicon.ico',
    1,
    0,
    0,
    1,
    20,
    CAST(strftime('%s', 'now', '-5 hours') AS INTEGER),
    18,
    3,
    '{"language": "en", "type": "database"}'
),
(
    204,
    2,
    104,
    'WHO',
    'https://www.who.int/',
    'World Health Organization - directing and coordinating international health.',
    'https://www.who.int/favicon.ico',
    0,
    0,
    0,
    1,
    12,
    CAST(strftime('%s', 'now', '-1 day') AS INTEGER),
    15,
    2,
    '{"language": "en", "type": "organization"}'
),
-- Finance bookmarks
(
    205,
    2,
    105,
    'Bloomberg',
    'https://www.bloomberg.com/',
    'Business, financial and economic news, and analysis.',
    'https://www.bloomberg.com/favicon.ico',
    1,
    0,
    0,
    1,
    18,
    CAST(strftime('%s', 'now', '-4 hours') AS INTEGER),
    20,
    2,
    '{"language": "en", "type": "news"}'
),
(
    206,
    2,
    105,
    'Coinbase',
    'https://www.coinbase.com/',
    'Buy and sell cryptocurrency on the leading digital asset platform.',
    'https://www.coinbase.com/favicon.ico',
    0,
    0,
    0,
    0,
    8,
    CAST(strftime('%s', 'now', '-2 days') AS INTEGER),
    10,
    1,
    '{"language": "en", "type": "platform"}'
),
-- Climate bookmarks
(
    207,
    2,
    106,
    'NASA Climate Change',
    'https://climate.nasa.gov/',
    'NASA''s website for news and data on climate change.',
    'https://climate.nasa.gov/favicon.ico',
    1,
    0,
    0,
    1,
    22,
    CAST(strftime('%s', 'now', '-6 hours') AS INTEGER),
    25,
    2,
    '{"language": "en", "type": "research"}'
),
(
    208,
    2,
    106,
    'IPCC Reports',
    'https://www.ipcc.ch/reports/',
    'Intergovernmental Panel on Climate Change assessment reports.',
    'https://www.ipcc.ch/favicon.ico',
    0,
    0,
    0,
    0,
    10,
    CAST(strftime('%s', 'now', '-3 days') AS INTEGER),
    40,
    4,
    '{"language": "en", "type": "report"}'
);

-- Insert bookmark-tag relationships for John Doe's additional bookmarks
INSERT INTO bookmark_tags (bookmark_id, tag_id) VALUES
-- DevOps bookmarks
(100, 100), -- Docker Documentation -> docker
(100, 113), -- Docker Documentation -> automation
(101, 101), -- Kubernetes Documentation -> kubernetes
(101, 107), -- Kubernetes Documentation -> performance
(102, 100), -- GitHub Actions -> docker
(102, 113), -- GitHub Actions -> automation
(103, 101), -- Terraform -> kubernetes
(103, 113), -- Terraform -> automation
-- Mobile Development bookmarks
(104, 102), -- React Native -> react
(104, 109), -- React Native -> mobile
(105, 109), -- Flutter -> mobile
(106, 110), -- iOS Developer Documentation -> ios
(106, 109), -- iOS Developer Documentation -> mobile
(107, 111), -- Android Developers -> android
(107, 109), -- Android Developers -> mobile
-- Security bookmarks
(108, 108), -- OWASP Top Ten -> security
(109, 108), -- Crash Course Computer Science -> security
-- Productivity bookmarks
(110, 112), -- Notion -> productivity
(111, 112), -- Obsidian -> productivity
-- More development bookmarks
(112, 2), -- Stack Overflow -> web-development
(113, 2), -- GitHub -> web-development
(114, 103), -- Vue.js Guide -> vue
(114, 2), -- Vue.js Guide -> web-development
(115, 104), -- TypeScript Handbook -> typescript
(115, 2); -- TypeScript Handbook -> web-development

-- Insert bookmark-tag relationships for Jane Smith's additional bookmarks
INSERT INTO bookmark_tags (bookmark_id, tag_id) VALUES
-- Research bookmarks
(200, 5), -- Nature -> machine-learning
(200, 6), -- Nature -> data-science
(200, 121), -- Nature -> statistics
(201, 5), -- Science Magazine -> machine-learning
(201, 6), -- Science Magazine -> data-science
(202, 5), -- Google Scholar -> machine-learning
(202, 6), -- Google Scholar -> data-science
-- Healthcare bookmarks
(203, 118), -- PubMed -> healthcare
(203, 6), -- PubMed -> data-science
(204, 118), -- WHO -> healthcare
-- Finance bookmarks
(205, 119), -- Bloomberg -> fintech
(206, 119), -- Coinbase -> fintech
-- Climate bookmarks
(207, 120), -- NASA Climate Change -> climate
(208, 120), -- IPCC Reports -> climate
(208, 121); -- IPCC Reports -> statistics

-- Update tag usage counts for all new tags
UPDATE tags SET usage_count = (
    SELECT COUNT(*)
    FROM bookmark_tags bt
    JOIN bookmarks b ON bt.bookmark_id = b.id
    WHERE bt.tag_id = tags.id AND b.is_archived = 0
);

-- Update collection bookmark counts for all new collections
UPDATE collections SET bookmark_count = (
    SELECT COUNT(*)
    FROM bookmarks
    WHERE collection_id = collections.id AND is_archived = 0
);

-- Note: bookmark_stats table has been removed, statistics will be calculated on-demand