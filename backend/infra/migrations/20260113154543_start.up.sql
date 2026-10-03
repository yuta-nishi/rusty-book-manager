-- updated_atを自動更新する
CREATE OR REPLACE FUNCTION set_updated_at() RETURNS trigger AS '
    BEGIN
        new.updated_at := ''now'';
        return new;
    END;
' LANGUAGE 'plpgsql' ;

-- rolesテーブルを作成する
CREATE TABLE IF NOT EXISTS roles (
role_id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
name varchar (255) NOT NULL UNIQUE
) ;

-- usersテーブルを作成する
CREATE TABLE IF NOT EXISTS users (
user_id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
name varchar (255) NOT NULL,
email varchar (255) NOT NULL UNIQUE,
password_hash varchar (255) NOT NULL,
role_id UUID NOT NULL,
created_at timestamp (3) WITH time zone NOT NULL DEFAULT CURRENT_TIMESTAMP (3),
updated_at timestamp (3) WITH time zone NOT NULL DEFAULT CURRENT_TIMESTAMP (3),

FOREIGN KEY (role_id) REFERENCES roles (role_id)
ON UPDATE CASCADE
ON DELETE CASCADE
) ;

-- usersテーブルにトリガーを追加する
CREATE TRIGGER users_updated_at_trigger
BEFORE UPDATE ON users FOR EACH ROW EXECUTE PROCEDURE set_updated_at () ;

-- booksテーブルを作成する
CREATE TABLE IF NOT EXISTS books (
book_id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
title varchar (255) NOT NULL,
author varchar (255) NOT NULL,
isbn varchar (255) NOT NULL,
description text NOT NULL,
user_id UUID NOT NULL,
created_at timestamp (3) WITH time zone NOT NULL DEFAULT CURRENT_TIMESTAMP (3),
updated_at timestamp (3) WITH time zone NOT NULL DEFAULT CURRENT_TIMESTAMP (3),

FOREIGN KEY (user_id) REFERENCES users (user_id)
ON UPDATE CASCADE
ON DELETE CASCADE
) ;

-- booksテーブルにトリガーを追加する
CREATE TRIGGER books_updated_at_trigger
BEFORE UPDATE ON books FOR EACH ROW EXECUTE PROCEDURE set_updated_at () ;

-- checkoutsテーブルを作成する
CREATE TABLE IF NOT EXISTS checkouts (
checkout_id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
book_id UUID NOT NULL UNIQUE,
user_id UUID NOT NULL,
checked_out_at timestamp (3) WITH time zone NOT NULL DEFAULT CURRENT_TIMESTAMP (3),

FOREIGN KEY (book_id) REFERENCES books (book_id)
ON UPDATE CASCADE
ON DELETE CASCADE,
FOREIGN KEY (user_id) REFERENCES users (user_id)
ON UPDATE CASCADE
ON DELETE CASCADE
) ;

-- returned_checkoutsテーブルを作成する
CREATE TABLE IF NOT EXISTS returned_checkouts (
checkout_id UUID PRIMARY KEY,
book_id UUID NOT NULL,
user_id UUID NOT NULL,
checked_out_at timestamp (3) WITH time zone NOT NULL DEFAULT CURRENT_TIMESTAMP (3),
returned_at timestamp (3) WITH time zone NOT NULL DEFAULT CURRENT_TIMESTAMP (3)
) ;
