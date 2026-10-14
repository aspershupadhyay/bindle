-- SPDX-License-Identifier: Apache-2.0
-- Copyright 2026 Sparsh Upadhyay and the Bindle contributors.
-- Migration 0004: keep the search index in step with promises (handbook P6, search).
-- Each promise's index row shares its rowid, so updates touch one row, not a scan.
-- The body holds the first step, the note and the person's name.

CREATE TRIGGER promise_search_insert AFTER INSERT ON promise BEGIN
  INSERT INTO search_index (rowid, ref_id, ref_kind, title, body)
  VALUES (new.rowid, new.id, 'promise', new.title,
          coalesce(new.first_step, '') || ' ' || coalesce(new.note, '') || ' ' ||
          coalesce((SELECT display_name FROM person WHERE id = new.person_id), ''));
END;

CREATE TRIGGER promise_search_update AFTER UPDATE OF title, first_step, note, person_id ON promise BEGIN
  DELETE FROM search_index WHERE rowid = old.rowid;
  INSERT INTO search_index (rowid, ref_id, ref_kind, title, body)
  VALUES (new.rowid, new.id, 'promise', new.title,
          coalesce(new.first_step, '') || ' ' || coalesce(new.note, '') || ' ' ||
          coalesce((SELECT display_name FROM person WHERE id = new.person_id), ''));
END;

CREATE TRIGGER promise_search_delete AFTER DELETE ON promise BEGIN
  DELETE FROM search_index WHERE rowid = old.rowid;
END;

CREATE TRIGGER person_search_rename AFTER UPDATE OF display_name ON person BEGIN
  DELETE FROM search_index WHERE rowid IN (SELECT rowid FROM promise WHERE person_id = new.id);
  INSERT INTO search_index (rowid, ref_id, ref_kind, title, body)
  SELECT p.rowid, p.id, 'promise', p.title,
         coalesce(p.first_step, '') || ' ' || coalesce(p.note, '') || ' ' || new.display_name
  FROM promise p WHERE p.person_id = new.id;
END;

-- Fill the index for promises saved before this migration.
DELETE FROM search_index;
INSERT INTO search_index (rowid, ref_id, ref_kind, title, body)
SELECT p.rowid, p.id, 'promise', p.title,
       coalesce(p.first_step, '') || ' ' || coalesce(p.note, '') || ' ' || coalesce(person.display_name, '')
FROM promise p LEFT JOIN person ON person.id = p.person_id;
