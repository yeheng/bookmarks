-- Trigger to update collection bookmark count
CREATE TRIGGER update_collection_bookmark_count_insert
    AFTER INSERT ON bookmarks
    FOR EACH ROW BEGIN
        UPDATE collections 
        SET bookmark_count = bookmark_count + 1 
        WHERE id = NEW.collection_id;
    END;

CREATE TRIGGER update_collection_bookmark_count_update
    AFTER UPDATE ON bookmarks
    FOR EACH ROW BEGIN
        IF OLD.collection_id IS NOT NEW.collection_id THEN
            IF OLD.collection_id IS NOT NULL THEN
                UPDATE collections 
                SET bookmark_count = bookmark_count - 1 
                WHERE id = OLD.collection_id;
            END IF;
            IF NEW.collection_id IS NOT NULL THEN
                UPDATE collections 
                SET bookmark_count = bookmark_count + 1 
                WHERE id = NEW.collection_id;
            END IF;
        END IF;
    END;

CREATE TRIGGER update_collection_bookmark_count_delete
    AFTER DELETE ON bookmarks
    FOR EACH ROW BEGIN
        IF OLD.collection_id IS NOT NULL THEN
            UPDATE collections 
            SET bookmark_count = bookmark_count - 1 
            WHERE id = OLD.collection_id;
        END IF;
    END;

-- Trigger to update tag usage count
CREATE TRIGGER update_tag_usage_count_insert
    AFTER INSERT ON bookmark_tags
    FOR EACH ROW BEGIN
        UPDATE tags 
        SET usage_count = usage_count + 1 
        WHERE id = NEW.tag_id;
    END;

CREATE TRIGGER update_tag_usage_count_delete
    AFTER DELETE ON bookmark_tags
    FOR EACH ROW BEGIN
        UPDATE tags 
        SET usage_count = usage_count - 1 
        WHERE id = OLD.tag_id;
    END;