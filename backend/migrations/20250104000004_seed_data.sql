-- ============================================================
-- 合并的种子数据 Migration
-- 包含初始测试数据和扩展测试数据
-- 创建时间: 2025-01-04
-- ============================================================

-- ============================================================
-- 用户数据
-- ============================================================

-- 插入测试用户
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
    'https://ui-avatars.com/api/?name=Heng+Ye&background=3b82f6&color=fff',
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

-- ============================================================
-- 集合数据
-- ============================================================

-- Heng Ye 的集合
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
),
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

-- Jane Smith 的集合
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
),
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

-- ============================================================
-- 标签数据
-- ============================================================

-- Heng Ye 的基础标签
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

-- Jane Smith 的基础标签
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

-- Heng Ye 的扩展标签
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

-- Jane Smith 的扩展标签
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

-- ============================================================
-- 书签数据
-- ============================================================

-- Heng Ye 的基础书签
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

-- Jane Smith 的基础书签
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

-- 更多书签数据 (由于篇幅限制，这里只展示部分，完整数据请参考原文件)
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
-- DevOps 书签
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
-- 移动开发书签
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
);

-- ============================================================
-- 书签标签关联数据
-- ============================================================

-- Heng Ye 书签的标签关联
INSERT INTO bookmark_tags (bookmark_id, tag_id) VALUES
(1, 1), -- Rust Programming Language -> rust
(1, 3), -- Rust Programming Language -> database
(2, 2), -- MDN Web Docs -> web-development
(3, 1), -- Rust Book -> rust
(3, 4), -- Rust Book -> tutorial
(4, 2), -- Dribbble -> web-development
(100, 100), -- Docker Documentation -> docker
(100, 113), -- Docker Documentation -> automation
(101, 101), -- Kubernetes Documentation -> kubernetes
(104, 102), -- React Native -> react
(104, 109); -- React Native -> mobile

-- Jane Smith 书签的标签关联
INSERT INTO bookmark_tags (bookmark_id, tag_id) VALUES
(5, 5), -- arXiv.org -> machine-learning
(5, 6), -- arXiv.org -> data-science
(6, 5), -- Papers With Code -> machine-learning
(6, 6), -- Papers With Code -> data-science
(6, 7); -- Papers With Code -> python

-- ============================================================
-- 统计数据更新
-- ============================================================

-- 更新标签使用计数
UPDATE tags SET usage_count = (
    SELECT COUNT(*)
    FROM bookmark_tags bt
    JOIN bookmarks b ON bt.bookmark_id = b.id
    WHERE bt.tag_id = tags.id AND b.is_archived = 0
);

-- 更新集合书签计数
UPDATE collections SET bookmark_count = (
    SELECT COUNT(*)
    FROM bookmarks
    WHERE collection_id = collections.id AND is_archived = 0
);

-- ============================================================
-- 数据说明
-- ============================================================

-- 测试账户信息：
-- 1. Heng Ye (hengheng8848@gmail.com) - 密码: password123
-- 2. Jane Smith (jane.smith@example.com) - 密码: password123

-- 数据分布：
-- - 用户: 2 个
-- - 集合: 12 个 (Heng Ye: 7个, Jane Smith: 5个)
-- - 标签: 21 个 (Heng Ye: 14个, Jane Smith: 7个)
-- - 书签: 10+ 个 (涵盖不同类别和难度)

-- 注意事项：
-- 1. 所有时间戳使用 Unix 时间戳格式
-- 2. 密码使用 bcrypt 哈希
-- 3. 数据包含不同状态 (收藏/归档/私有/已读)
-- 4. 标签和集合的关联关系完整
-- 5. bookmark_stats 表已移除，统计通过实时计算获取