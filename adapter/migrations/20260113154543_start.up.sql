-- updated_atを自動更新する
CREATE OR REPLACE FUNCTION set_updated_at() RETURNS trigger AS '
    BEGIN
        new.updated_at := ''now'';
        return new;
    END;
' LANGUAGE 'plpgsql' ;

-- booksテーブルを作成する
CREATE TABLE IF NOT EXISTS books (
book_id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
title varchar (255) NOT NULL,
author varchar (255) NOT NULL,
isbn varchar (255) NOT NULL,
description text NOT NULL,
created_at timestamp (3) WITH time zone NOT NULL DEFAULT CURRENT_TIMESTAMP (3),
updated_at timestamp (3) WITH time zone NOT NULL DEFAULT CURRENT_TIMESTAMP (3)
) ;

-- booksテーブルにトリガーを追加する
CREATE TRIGGER set_updated_at_books
BEFORE UPDATE ON books FOR EACH ROW EXECUTE PROCEDURE set_updated_at () ;
