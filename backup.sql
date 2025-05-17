--
-- PostgreSQL database cluster dump
--

SET default_transaction_read_only = off;

SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;

--
-- Roles
--

CREATE ROLE postgres;
ALTER ROLE postgres WITH SUPERUSER INHERIT CREATEROLE CREATEDB LOGIN REPLICATION BYPASSRLS PASSWORD 'SCRAM-SHA-256$4096:0PF2jgKG7mESfFws9fYFLw==$mL7jO/QbrOSPIx6RmLjZdAyCL7VyCi5firFgEick60I=:KCIVlO6Me4AV/Upn7D2cbTL4D7qQ4oAh0R6pZtHGFn4=';

--
-- User Configurations
--








--
-- Databases
--

--
-- Database "template1" dump
--

\connect template1

--
-- PostgreSQL database dump
--

-- Dumped from database version 17.2 (Debian 17.2-1.pgdg120+1)
-- Dumped by pg_dump version 17.2 (Debian 17.2-1.pgdg120+1)

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET transaction_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- PostgreSQL database dump complete
--

--
-- Database "postgres" dump
--

\connect postgres

--
-- PostgreSQL database dump
--

-- Dumped from database version 17.2 (Debian 17.2-1.pgdg120+1)
-- Dumped by pg_dump version 17.2 (Debian 17.2-1.pgdg120+1)

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET transaction_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: users; Type: SCHEMA; Schema: -; Owner: postgres
--

CREATE SCHEMA users;


ALTER SCHEMA users OWNER TO postgres;

--
-- Name: update_timestamp(); Type: FUNCTION; Schema: users; Owner: postgres
--

CREATE FUNCTION users.update_timestamp() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$;


ALTER FUNCTION users.update_timestamp() OWNER TO postgres;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: _sqlx_migrations; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public._sqlx_migrations (
    version bigint NOT NULL,
    description text NOT NULL,
    installed_on timestamp with time zone DEFAULT now() NOT NULL,
    success boolean NOT NULL,
    checksum bytea NOT NULL,
    execution_time bigint NOT NULL
);


ALTER TABLE public._sqlx_migrations OWNER TO postgres;

--
-- Name: appointments; Type: TABLE; Schema: users; Owner: postgres
--

CREATE TABLE users.appointments (
    id bigint NOT NULL,
    user_id bigint NOT NULL,
    receptionist_id bigint,
    technician_id bigint,
    start_time character varying(30) NOT NULL,
    end_time character varying(30),
    updated_by bigint,
    status character varying(20) DEFAULT 'PENDING'::character varying NOT NULL,
    notes text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT appointments_status_check CHECK (((status)::text = ANY ((ARRAY['PENDING'::character varying, 'CONFIRMED'::character varying, 'IN_PROGRESS'::character varying, 'COMPLETED'::character varying, 'CANCELLED'::character varying])::text[])))
);


ALTER TABLE users.appointments OWNER TO postgres;

--
-- Name: appointments_id_seq; Type: SEQUENCE; Schema: users; Owner: postgres
--

CREATE SEQUENCE users.appointments_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE users.appointments_id_seq OWNER TO postgres;

--
-- Name: appointments_id_seq; Type: SEQUENCE OWNED BY; Schema: users; Owner: postgres
--

ALTER SEQUENCE users.appointments_id_seq OWNED BY users.appointments.id;


--
-- Name: appointments_services; Type: TABLE; Schema: users; Owner: postgres
--

CREATE TABLE users.appointments_services (
    id bigint NOT NULL,
    appointment_id bigint NOT NULL,
    service_id bigint NOT NULL,
    technician_id bigint,
    quantity integer DEFAULT 1,
    sequence integer DEFAULT 1,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_by bigint
);


ALTER TABLE users.appointments_services OWNER TO postgres;

--
-- Name: appointments_services_id_seq; Type: SEQUENCE; Schema: users; Owner: postgres
--

CREATE SEQUENCE users.appointments_services_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE users.appointments_services_id_seq OWNER TO postgres;

--
-- Name: appointments_services_id_seq; Type: SEQUENCE OWNED BY; Schema: users; Owner: postgres
--

ALTER SEQUENCE users.appointments_services_id_seq OWNED BY users.appointments_services.id;


--
-- Name: chat_messages; Type: TABLE; Schema: users; Owner: postgres
--

CREATE TABLE users.chat_messages (
    id bigint NOT NULL,
    sender_id bigint NOT NULL,
    receiver_id bigint NOT NULL,
    message text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


ALTER TABLE users.chat_messages OWNER TO postgres;

--
-- Name: chat_messages_id_seq; Type: SEQUENCE; Schema: users; Owner: postgres
--

CREATE SEQUENCE users.chat_messages_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE users.chat_messages_id_seq OWNER TO postgres;

--
-- Name: chat_messages_id_seq; Type: SEQUENCE OWNED BY; Schema: users; Owner: postgres
--

ALTER SEQUENCE users.chat_messages_id_seq OWNED BY users.chat_messages.id;


--
-- Name: phone_codes; Type: TABLE; Schema: users; Owner: postgres
--

CREATE TABLE users.phone_codes (
    id bigint NOT NULL,
    user_id bigint NOT NULL,
    phone character varying(15) NOT NULL,
    code character varying(6) NOT NULL,
    revoked boolean DEFAULT false,
    last_used_at timestamp with time zone,
    expires_at timestamp with time zone NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT valid_expires CHECK ((expires_at > created_at))
);


ALTER TABLE users.phone_codes OWNER TO postgres;

--
-- Name: phone_codes_id_seq; Type: SEQUENCE; Schema: users; Owner: postgres
--

CREATE SEQUENCE users.phone_codes_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE users.phone_codes_id_seq OWNER TO postgres;

--
-- Name: phone_codes_id_seq; Type: SEQUENCE OWNED BY; Schema: users; Owner: postgres
--

ALTER SEQUENCE users.phone_codes_id_seq OWNED BY users.phone_codes.id;


--
-- Name: refresh_tokens; Type: TABLE; Schema: users; Owner: postgres
--

CREATE TABLE users.refresh_tokens (
    id bigint NOT NULL,
    user_id bigint NOT NULL,
    token text NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    revoked boolean DEFAULT false,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    last_used_at timestamp with time zone,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    device_info character varying(150),
    CONSTRAINT valid_expires CHECK ((expires_at > created_at))
);


ALTER TABLE users.refresh_tokens OWNER TO postgres;

--
-- Name: refresh_tokens_id_seq; Type: SEQUENCE; Schema: users; Owner: postgres
--

CREATE SEQUENCE users.refresh_tokens_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE users.refresh_tokens_id_seq OWNER TO postgres;

--
-- Name: refresh_tokens_id_seq; Type: SEQUENCE OWNED BY; Schema: users; Owner: postgres
--

ALTER SEQUENCE users.refresh_tokens_id_seq OWNED BY users.refresh_tokens.id;


--
-- Name: service_items; Type: TABLE; Schema: users; Owner: postgres
--

CREATE TABLE users.service_items (
    id bigint NOT NULL,
    parent_service_id bigint NOT NULL,
    service_name character varying(100) NOT NULL,
    image text,
    service_name_en text,
    service_name_ko text,
    description text,
    service_type character varying(255),
    description_en text,
    description_ko text,
    price integer NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT service_items_price_check CHECK ((price >= 0))
);


ALTER TABLE users.service_items OWNER TO postgres;

--
-- Name: service_items_id_seq; Type: SEQUENCE; Schema: users; Owner: postgres
--

CREATE SEQUENCE users.service_items_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE users.service_items_id_seq OWNER TO postgres;

--
-- Name: service_items_id_seq; Type: SEQUENCE OWNED BY; Schema: users; Owner: postgres
--

ALTER SEQUENCE users.service_items_id_seq OWNED BY users.service_items.id;


--
-- Name: services; Type: TABLE; Schema: users; Owner: postgres
--

CREATE TABLE users.services (
    id bigint NOT NULL,
    service_name character varying(100) NOT NULL,
    description text,
    price integer NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    image text,
    service_type character varying(255),
    service_name_en text,
    service_name_ko text,
    description_ko text,
    description_en text,
    is_category boolean,
    has_child boolean DEFAULT false,
    CONSTRAINT services_price_check CHECK ((price >= 0))
);


ALTER TABLE users.services OWNER TO postgres;

--
-- Name: services_id_seq; Type: SEQUENCE; Schema: users; Owner: postgres
--

CREATE SEQUENCE users.services_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE users.services_id_seq OWNER TO postgres;

--
-- Name: services_id_seq; Type: SEQUENCE OWNED BY; Schema: users; Owner: postgres
--

ALTER SEQUENCE users.services_id_seq OWNED BY users.services.id;


--
-- Name: tbl_users; Type: TABLE; Schema: users; Owner: postgres
--

CREATE TABLE users.tbl_users (
    pk_user_id bigint NOT NULL,
    user_name character varying(150),
    phone character varying(15),
    full_name character varying(150),
    role text DEFAULT 'USER'::text NOT NULL,
    password_hash character varying(150),
    email_address character varying(150),
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    is_verify boolean DEFAULT false NOT NULL,
    date_of_birth character varying(50),
    address character varying(200),
    avatar character varying(255),
    CONSTRAINT check_user_credentials CHECK ((((role = ANY (ARRAY['ADMIN'::text, 'USER'::text])) AND (user_name IS NOT NULL) AND (password_hash IS NOT NULL)) OR ((role <> ALL (ARRAY['ADMIN'::text, 'USER'::text])) AND (phone IS NOT NULL))))
);


ALTER TABLE users.tbl_users OWNER TO postgres;

--
-- Name: tbl_users_pk_user_id_seq; Type: SEQUENCE; Schema: users; Owner: postgres
--

CREATE SEQUENCE users.tbl_users_pk_user_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE users.tbl_users_pk_user_id_seq OWNER TO postgres;

--
-- Name: tbl_users_pk_user_id_seq; Type: SEQUENCE OWNED BY; Schema: users; Owner: postgres
--

ALTER SEQUENCE users.tbl_users_pk_user_id_seq OWNED BY users.tbl_users.pk_user_id;


--
-- Name: appointments id; Type: DEFAULT; Schema: users; Owner: postgres
--

ALTER TABLE ONLY users.appointments ALTER COLUMN id SET DEFAULT nextval('users.appointments_id_seq'::regclass);


--
-- Name: appointments_services id; Type: DEFAULT; Schema: users; Owner: postgres
--

ALTER TABLE ONLY users.appointments_services ALTER COLUMN id SET DEFAULT nextval('users.appointments_services_id_seq'::regclass);


--
-- Name: chat_messages id; Type: DEFAULT; Schema: users; Owner: postgres
--

ALTER TABLE ONLY users.chat_messages ALTER COLUMN id SET DEFAULT nextval('users.chat_messages_id_seq'::regclass);


--
-- Name: phone_codes id; Type: DEFAULT; Schema: users; Owner: postgres
--

ALTER TABLE ONLY users.phone_codes ALTER COLUMN id SET DEFAULT nextval('users.phone_codes_id_seq'::regclass);


--
-- Name: refresh_tokens id; Type: DEFAULT; Schema: users; Owner: postgres
--

ALTER TABLE ONLY users.refresh_tokens ALTER COLUMN id SET DEFAULT nextval('users.refresh_tokens_id_seq'::regclass);


--
-- Name: service_items id; Type: DEFAULT; Schema: users; Owner: postgres
--

ALTER TABLE ONLY users.service_items ALTER COLUMN id SET DEFAULT nextval('users.service_items_id_seq'::regclass);


--
-- Name: services id; Type: DEFAULT; Schema: users; Owner: postgres
--

ALTER TABLE ONLY users.services ALTER COLUMN id SET DEFAULT nextval('users.services_id_seq'::regclass);


--
-- Name: tbl_users pk_user_id; Type: DEFAULT; Schema: users; Owner: postgres
--

ALTER TABLE ONLY users.tbl_users ALTER COLUMN pk_user_id SET DEFAULT nextval('users.tbl_users_pk_user_id_seq'::regclass);


--
-- Data for Name: _sqlx_migrations; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public._sqlx_migrations (version, description, installed_on, success, checksum, execution_time) FROM stdin;
11	add table service item	2025-05-11 06:29:59.269634+00	t	\\x3545d127d8dbf8b48f35f1d07a0023fac48305a5278fa082df22a7117471f23b20a9154fd4418237bb9ab153c80002f2	14363708
12	add table appointments	2025-05-13 03:41:20.132913+00	t	\\x228a668e96002fbccf03843fac21e8934377ad7db285c2cc8de015e18ee263928ee14aed81be1b10702dead81d69a3f7	16622375
1	initiallized	2025-04-12 04:28:07.271687+00	t	\\xea1b5addecf15fc1309e12e00f9a72bc13471c002327b463d091ef0f8cf616d216f03877c314e4fa8e607b648307700f	19403250
2	update token table	2025-04-12 04:28:07.2938+00	t	\\x995fa14215c17595ffe0a7cba798c26b5a006a5326a7979172f3094af4be4cbd2126028d6d975df7eef99742fef31e43	3011500
3	create chat message	2025-04-12 04:28:07.297581+00	t	\\xa0acbeeaa3d6df4d4c1ae26a698a8a8f39266dd8b52104c2a9a8c2c5f3cf82ca841d0e5c52b02eb008497edbaf14f257	4840959
4	create phone code table	2025-04-12 04:28:07.30306+00	t	\\xa5e7af06905677a56d3eb17191559847f505b9089cbbd81d6f5b240b6209587ed6525c60a52df2fe9f00410969951915	3265166
5	add columns address	2025-04-27 18:20:36.818887+00	t	\\xc47730854c7db02ad4f9f0011ff7caa6e28632ba51309cddb91330b21b44fa8393139fb84857766fc000ba49d022bb9e	8693500
7	add trigger update time	2025-04-28 10:15:39.847953+00	t	\\x19e6e7909abe5f8db0db4e8fc783117b22809c20469d2e1eb23892974c78b0b8d9429ca058946ec7fc724af2a8c04ebb	3701708
8	add columns image	2025-05-01 09:12:35.591569+00	t	\\x14c91e88c8e2ad8d8f9cc6f75969a9a686f883eef7dcdc2c98b3bb6611de189726734cb74664f98460408565472c1b74	9553542
6	add table service	2025-05-04 12:32:19.902554+00	t	\\x715e4fd0e3ab286c7a5a166d6f5cbd364297fbf8baabac02cf950fb1435f54e09eaa1da3150ca3d1524ffa9374209996	7691750
9	add columns image service	2025-05-04 12:32:19.911441+00	t	\\x3717619fa60bd66b65b190977ed545a60f96b53f3a83d4b9786e4f503233f16214e28bd767939f95b75de4eb756fb64e	1839208
10	add column language service	2025-05-11 06:23:49.985091+00	t	\\x33f129b1d66e48ae84ecca93667366ced981c6206114d8b78c95d54a2f856643040c69197ce5063871f67ddf48196c5d	5285542
\.


--
-- Data for Name: appointments; Type: TABLE DATA; Schema: users; Owner: postgres
--

COPY users.appointments (id, user_id, receptionist_id, technician_id, start_time, end_time, updated_by, status, notes, created_at, updated_at) FROM stdin;
4	9	\N	\N	9:00 13/6/2025	\N	1	PENDING	\N	2025-05-13 15:52:17.588483+00	2025-05-14 04:21:17.885295+00
5	9	\N	\N	9:00 13/6/2025	\N	9	PENDING	\N	2025-05-15 22:06:37.956972+00	2025-05-15 22:06:37.956972+00
6	9	\N	\N	06:25 16/05/2025	\N	9	PENDING	Ssdsdsds	2025-05-15 22:26:14.666772+00	2025-05-15 22:26:14.666772+00
7	9	\N	\N	07:29 16/05/2025	\N	9	PENDING	Sdsdsds	2025-05-15 22:29:36.712521+00	2025-05-15 22:29:36.712521+00
8	9	\N	\N	06:37 16/05/2025	\N	9	PENDING		2025-05-15 22:37:59.866076+00	2025-05-15 22:37:59.866076+00
9	9	\N	\N	06:50 16/05/2025	\N	9	PENDING	Sdsdsd	2025-05-15 22:38:50.902879+00	2025-05-15 22:38:50.902879+00
10	9	\N	\N	19:43 16/05/2025	\N	9	PENDING		2025-05-15 22:43:43.167348+00	2025-05-15 22:43:43.167348+00
11	9	\N	\N	07:34 16/05/2025	\N	9	PENDING		2025-05-15 22:44:21.386203+00	2025-05-15 22:44:21.386203+00
\.


--
-- Data for Name: appointments_services; Type: TABLE DATA; Schema: users; Owner: postgres
--

COPY users.appointments_services (id, appointment_id, service_id, technician_id, quantity, sequence, created_at, updated_at, updated_by) FROM stdin;
2	4	4	\N	1	1	2025-05-13 15:52:17.59525+00	2025-05-13 15:52:17.59525+00	\N
3	4	5	\N	1	1	2025-05-13 15:52:17.599471+00	2025-05-13 15:52:17.599471+00	\N
4	4	6	\N	1	1	2025-05-13 15:52:17.602351+00	2025-05-13 15:52:17.602351+00	\N
5	5	4	\N	1	1	2025-05-15 22:06:37.956972+00	2025-05-15 22:06:37.956972+00	9
6	5	5	\N	1	1	2025-05-15 22:06:37.956972+00	2025-05-15 22:06:37.956972+00	9
7	6	6	\N	1	1	2025-05-15 22:26:14.666772+00	2025-05-15 22:26:14.666772+00	9
8	6	7	\N	1	1	2025-05-15 22:26:14.666772+00	2025-05-15 22:26:14.666772+00	9
9	7	7	\N	1	1	2025-05-15 22:29:36.712521+00	2025-05-15 22:29:36.712521+00	9
10	7	6	\N	1	1	2025-05-15 22:29:36.712521+00	2025-05-15 22:29:36.712521+00	9
11	8	7	\N	1	1	2025-05-15 22:37:59.866076+00	2025-05-15 22:37:59.866076+00	9
12	9	7	\N	1	1	2025-05-15 22:38:50.902879+00	2025-05-15 22:38:50.902879+00	9
13	10	8	\N	1	1	2025-05-15 22:43:43.167348+00	2025-05-15 22:43:43.167348+00	9
14	11	8	\N	1	1	2025-05-15 22:44:21.386203+00	2025-05-15 22:44:21.386203+00	9
15	11	9	\N	1	1	2025-05-15 22:44:21.386203+00	2025-05-15 22:44:21.386203+00	9
\.


--
-- Data for Name: chat_messages; Type: TABLE DATA; Schema: users; Owner: postgres
--

COPY users.chat_messages (id, sender_id, receiver_id, message, created_at) FROM stdin;
\.


--
-- Data for Name: phone_codes; Type: TABLE DATA; Schema: users; Owner: postgres
--

COPY users.phone_codes (id, user_id, phone, code, revoked, last_used_at, expires_at, created_at) FROM stdin;
55	12	+84961483890	222647	f	\N	2025-04-24 23:18:34.74431+00	2025-04-24 23:16:34.744711+00
56	13	+84961483900	455043	f	\N	2025-04-24 23:40:11.380136+00	2025-04-24 23:38:11.379681+00
\.


--
-- Data for Name: refresh_tokens; Type: TABLE DATA; Schema: users; Owner: postgres
--

COPY users.refresh_tokens (id, user_id, token, expires_at, revoked, created_at, last_used_at, updated_at, device_info) FROM stdin;
14	9	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI5Iiwicm9sZSI6IkNVU1RPTUVSIiwiZXhwIjoxNzQ2MTQwMTc0fQ.gLmOyFZOrqS7iCTHf-TmqEvh7SoJKCY0ROe5AokD3Z0	2025-05-01 22:56:14.042345+00	f	2025-04-24 22:56:14.042407+00	\N	2025-04-24 22:56:14.042407+00	\N
15	9	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI5Iiwicm9sZSI6IkNVU1RPTUVSIiwiZXhwIjoxNzQ2MTQ0Njg2fQ.a6Ishdnszh2kzUC2QCqRstFvrJlqdhad8UAr-Gg2sec	2025-05-02 00:11:26.733075+00	f	2025-04-25 00:11:26.735564+00	\N	2025-04-25 00:11:26.735564+00	\N
16	9	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI5Iiwicm9sZSI6IkNVU1RPTUVSIiwiZXhwIjoxNzQ2MTQ0NzczfQ.CXurTU2HRnRv8y0-cAO3NC8-uaU_wSvH94pUF1RsFE8	2025-05-02 00:12:53.92127+00	f	2025-04-25 00:12:53.923221+00	\N	2025-04-25 00:12:53.923221+00	\N
17	9	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI5Iiwicm9sZSI6IkNVU1RPTUVSIiwiZXhwIjoxNzQ2MTcxMjA0fQ.01cw_ogsoEDYlmZre0Sh_bNshwPegudn27aUMFmcC8w	2025-05-02 07:33:24.827953+00	f	2025-04-25 07:33:24.88683+00	\N	2025-04-25 07:33:24.88683+00	\N
18	9	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI5Iiwicm9sZSI6IkNVU1RPTUVSIiwiZXhwIjoxNzQ2MTc3ODIwfQ._fSHC8apd3JjJH3fr6nFgXEiyCEuSwVIOtJmRnTM7M0	2025-05-02 09:23:40.947545+00	f	2025-04-25 09:23:40.948383+00	\N	2025-04-25 09:23:40.948383+00	\N
19	9	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI5Iiwicm9sZSI6IkNVU1RPTUVSIiwiZXhwIjoxNzQ2MTk0NTQwfQ.2BlPNgGvY3w5eeV_0GAvfraMo-eD5XnJoCNVdY0wOks	2025-05-02 14:02:20.635203+00	f	2025-04-25 14:02:20.715871+00	\N	2025-04-25 14:02:20.715871+00	\N
20	9	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI5Iiwicm9sZSI6IkNVU1RPTUVSIiwiZXhwIjoxNzQ2MTk1Mzg0fQ._j1p7Tw_2apLr5iFDe8k-nH8Yod09cg7mt0WQU2o234	2025-05-02 14:16:24.695319+00	f	2025-04-25 14:16:24.701194+00	\N	2025-04-25 14:16:24.701194+00	\N
21	9	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI5Iiwicm9sZSI6IkNVU1RPTUVSIiwiZXhwIjoxNzQ2MTk3MjkxfQ.GqIiznZ6rL92MEQHfgFNysSirAcn5g7_ALnE6Hs3B0Y	2025-05-02 14:48:11.343594+00	f	2025-04-25 14:48:11.413403+00	\N	2025-04-25 14:48:11.413403+00	\N
22	9	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI5Iiwicm9sZSI6IkNVU1RPTUVSIiwiZXhwIjoxNzQ2MjAyMzc1fQ.df5nuRwP7AShgGCLU7karKb8kDPg3u9k8z7ceX3moJw	2025-05-02 16:12:55.5833+00	f	2025-04-25 16:12:55.651649+00	\N	2025-04-25 16:12:55.651649+00	\N
23	1	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIiwicm9sZSI6IkFETUlOIiwiZXhwIjoxNzQ2MjYxNDk4fQ.Cbfi5M51eVICJUvQsIR-LutzCzA3XAUh0cSgCwc2wKY	2025-05-03 08:38:18.899471+00	f	2025-04-26 08:38:18.977815+00	\N	2025-04-26 08:38:18.977815+00	\N
25	1	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIiwicm9sZSI6IkFETUlOIiwiZXhwIjoxNzQ2MjgyMTU0fQ.UcRrXjIcjlfLVsw5LVVARkr3rVrSY0kDZfyXw0Ced74	2025-05-03 14:22:34.007016+00	f	2025-04-26 14:22:34.078942+00	\N	2025-04-26 14:22:34.078942+00	\N
27	9	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI5Iiwicm9sZSI6IkNVU1RPTUVSIiwiZXhwIjoxNzQ2Mjg0MjAzfQ.0b49DLgQSqbv3BORoURyd1O3b8vLRkRZiub-N6UF_2w	2025-05-03 14:56:43.761357+00	f	2025-04-26 14:56:44.100615+00	\N	2025-04-26 14:56:44.100615+00	\N
30	1	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIiwicm9sZSI6IkFETUlOIiwiZXhwIjoxNzQ2MzQzNjYzfQ.V1AEqUuaUHCjlKVFrPeCDze6_PEqkO5a_Tqo3vR5lSY	2025-05-04 07:27:43.859037+00	f	2025-04-27 07:27:43.922718+00	\N	2025-04-27 07:27:43.922718+00	\N
31	1	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIiwicm9sZSI6IkFETUlOIiwiZXhwIjoxNzQ2MzQzOTYwfQ.FgehHbFSeIwDJ80E5ixF_Z4Ro6vv6T-YQXkrehrUt30	2025-05-04 07:32:40.020314+00	f	2025-04-27 07:32:40.091737+00	\N	2025-04-27 07:32:40.091737+00	\N
32	1	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIiwicm9sZSI6IkFETUlOIiwiZXhwIjoxNzQ2MzQ1NDU4fQ.zLkvNGUAWTK_-OisHMbo_qQwyq1iFPGuAv5gH6C-vLE	2025-05-04 07:57:38.69018+00	f	2025-04-27 07:57:38.760764+00	\N	2025-04-27 07:57:38.760764+00	\N
33	1	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIiwicm9sZSI6IkFETUlOIiwiZXhwIjoxNzQ2MzQ1NTQ5fQ.3XiR-A_wpsD7gKIxiu27kZG1vrzYFqsB7XT3fW8NPaw	2025-05-04 07:59:09.833443+00	f	2025-04-27 07:59:09.838904+00	\N	2025-04-27 07:59:09.838904+00	\N
36	9	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI5Iiwicm9sZSI6IkNVU1RPTUVSIiwiZXhwIjoxNzQ2MzcyMTM3fQ.LlEczkT8H6l5oXySmuiXPz5oYtpPTP9DkduOwZGN_tQ	2025-05-04 15:22:17.776268+00	f	2025-04-27 15:22:17.782621+00	\N	2025-04-27 15:22:17.782621+00	\N
38	1	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIiwicm9sZSI6IkFETUlOIiwiZXhwIjoxNzQ2NzE3NDYzfQ.Xw2hQ_kZZUy4gr7Am5YjuCV9jPmbJSXusxJ-LIsrRLM	2025-05-08 15:17:43.538315+00	f	2025-05-01 15:17:43.591618+00	\N	2025-05-01 15:17:43.591618+00	\N
39	1	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIiwicm9sZSI6IkFETUlOIiwiZXhwIjoxNzQ2NzE3NzgyfQ.h3Cbn-fqUoFBPfTDEvQr4F7NUt7fyC_8NkWR-vcE67U	2025-05-08 15:23:02.343311+00	f	2025-05-01 15:23:02.405086+00	\N	2025-05-01 15:23:02.405086+00	\N
40	1	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIiwicm9sZSI6IkFETUlOIiwiZXhwIjoxNzQ2NzE4NTcxfQ.3mNVZcamy0MfDvtLsMUX9GWhDK6e4EbC2fKqzsczE9k	2025-05-08 15:36:11.018419+00	f	2025-05-01 15:36:11.070504+00	\N	2025-05-01 15:36:11.070504+00	\N
41	1	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIiwicm9sZSI6IkFETUlOIiwiZXhwIjoxNzQ2NzIxNzE0fQ.a4buCX5vUP6SjpuWRTLhW6FgMUHPYdtC3HhEQgqfV30	2025-05-08 16:28:34.5227+00	f	2025-05-01 16:28:34.580216+00	\N	2025-05-01 16:28:34.580216+00	\N
42	9	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI5Iiwicm9sZSI6IkNVU1RPTUVSIiwiZXhwIjoxNzQ2NzgwNTMzfQ.zUtQYO7piYhetDwxM22ele_LsjQKcJ1gzLOn_UpSQQo	2025-05-09 08:48:53.971488+00	f	2025-05-02 08:48:54.06536+00	\N	2025-05-02 08:48:54.06536+00	\N
43	1	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIiwicm9sZSI6IkFETUlOIiwiZXhwIjoxNzQ2OTY1Mjk4fQ.kUdrE0FAazk7wf3sCpN694uPaioX1KSd6yWiJizO5_M	2025-05-11 12:08:18.222161+00	f	2025-05-04 12:08:18.284787+00	\N	2025-05-04 12:08:18.284787+00	\N
44	1	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIiwicm9sZSI6IkFETUlOIiwiZXhwIjoxNzQ3MTQ3MzY3fQ.hhtg3S6utpc1f-pIVi8rMp4FYRvCzFZcWhNF7-qzLCU	2025-05-13 14:42:47.830957+00	f	2025-05-06 14:42:47.898154+00	\N	2025-05-06 14:42:47.898154+00	\N
45	9	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI5Iiwicm9sZSI6IkNVU1RPTUVSIiwiZXhwIjoxNzQ3MTUxMzIwfQ.MKGmk5BVDbOY7tMf4ujMrqpx4XmhMyrhSbegkOQjrG8	2025-05-13 15:48:40.056405+00	f	2025-05-06 15:48:40.130962+00	\N	2025-05-06 15:48:40.130962+00	\N
46	9	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI5Iiwicm9sZSI6IkNVU1RPTUVSIiwiZXhwIjoxNzQ3MjA3OTY2fQ.OHeG53XQ3sI58L6JGdDgdowWpi8LyoFEHEtXxFPhusk	2025-05-14 07:32:46.02178+00	f	2025-05-07 07:32:46.080791+00	\N	2025-05-07 07:32:46.080791+00	\N
47	9	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI5Iiwicm9sZSI6IkNVU1RPTUVSIiwiZXhwIjoxNzQ3MjQ2NjgwfQ.Po7TOOmzXeOthQhqxdvKwF7cc7GlBU6LdKZlQh0a5JE	2025-05-14 18:18:00.400197+00	f	2025-05-07 18:18:00.475742+00	\N	2025-05-07 18:18:00.475742+00	\N
48	1	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIiwicm9sZSI6IkFETUlOIiwiZXhwIjoxNzQ3MzY4OTcwfQ.KiB_uWKqjoOJ1XMIudTt1dVTmkvgU3sflaGyhSn6Sis	2025-05-16 04:16:10.851109+00	f	2025-05-09 04:16:10.922051+00	\N	2025-05-09 04:16:10.922051+00	\N
49	1	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIiwicm9sZSI6IkFETUlOIiwiZXhwIjoxNzQ3NTQ5NDYxfQ.RUUXOUU6pEm5UsNnL_QC6YgfujNrsgvcEdVu7C-OaCM	2025-05-18 06:24:21.553709+00	f	2025-05-11 06:24:21.600871+00	\N	2025-05-11 06:24:21.600871+00	\N
50	1	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIiwicm9sZSI6IkFETUlOIiwiZXhwIjoxNzQ3NTQ5NDc2fQ.LYaRTW2TuM0_F2NMuyg0Ff8BunR2fFmOJXLlZIF3VHE	2025-05-18 06:24:36.657822+00	f	2025-05-11 06:24:36.659781+00	\N	2025-05-11 06:24:36.659781+00	\N
51	1	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIiwicm9sZSI6IkFETUlOIiwiZXhwIjoxNzQ3NTQ5NTI4fQ.zqnTvs37oyxwJ7gODq7rYtkm8HsbxXWHSyf4dMeH9Zw	2025-05-18 06:25:28.345659+00	f	2025-05-11 06:25:28.34846+00	\N	2025-05-11 06:25:28.34846+00	\N
52	1	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIiwicm9sZSI6IkFETUlOIiwiZXhwIjoxNzQ3NTQ5NTc2fQ.R03c1PZTJKHoSUZEdZV4yKBrdiv3pO7aJweRPGP-TRo	2025-05-18 06:26:16.752942+00	f	2025-05-11 06:26:16.75427+00	\N	2025-05-11 06:26:16.75427+00	\N
53	9	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI5Iiwicm9sZSI6IkNVU1RPTUVSIiwiZXhwIjoxNzQ3NTU4OTY5fQ.ELJ7J80Q6CG8BS1hompFjAbT3JUmDsX1luwMzVbNTNs	2025-05-18 09:02:49.759347+00	f	2025-05-11 09:02:49.819824+00	\N	2025-05-11 09:02:49.819824+00	\N
54	1	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIiwicm9sZSI6IkFETUlOIiwiZXhwIjoxNzQ3NzIzMTUxfQ.xMJMxLcLV3LEh-a1tp8O5ivBNGk0Zn6pls_LC8RL9oY	2025-05-20 06:39:11.253438+00	f	2025-05-13 06:39:11.31071+00	\N	2025-05-13 06:39:11.31071+00	\N
55	9	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI5Iiwicm9sZSI6IkNVU1RPTUVSIiwiZXhwIjoxNzQ3ODEzNTE0fQ.l9AL_6zKL2hDrRxM9cuRyB9z72xhi0nyLprRFshEZro	2025-05-21 07:45:14.794302+00	f	2025-05-14 07:45:14.7966+00	\N	2025-05-14 07:45:14.7966+00	\N
56	9	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI5Iiwicm9sZSI6IkNVU1RPTUVSIiwiZXhwIjoxNzQ3ODM5MjE4fQ.Rn2MF8qZKqo0nye6utOkHrIjWvz8ACXP0_9VoG8_tDs	2025-05-21 14:53:38.022622+00	f	2025-05-14 14:53:38.103924+00	\N	2025-05-14 14:53:38.103924+00	\N
61	9	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI5Iiwicm9sZSI6IkNVU1RPTUVSIiwiZXhwIjoxNzQ3OTMwNzUxfQ.rYgG2dRxzBvNTH81arlrSNteTi_MHjTl58Eh1iceTBQ	2025-05-22 16:19:11.480813+00	f	2025-05-15 16:19:11.487911+00	\N	2025-05-15 16:19:11.487911+00	\N
62	9	eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI5Iiwicm9sZSI6IkNVU1RPTUVSIiwiZXhwIjoxNzQ3OTM4Njc0fQ.P0cZBFVLd0GyFHGdITZ0NrXnV5f3c4GUNjFpC93TZFk	2025-05-22 18:31:14.052611+00	f	2025-05-15 18:31:14.153336+00	\N	2025-05-15 18:31:14.153336+00	\N
\.


--
-- Data for Name: service_items; Type: TABLE DATA; Schema: users; Owner: postgres
--

COPY users.service_items (id, parent_service_id, service_name, image, service_name_en, service_name_ko, description, service_type, description_en, description_ko, price, is_active, created_at, updated_at) FROM stdin;
6	1	Massage thuỵ điển	uploads/services/1-7b681016-becd-496a-807e-60d93327b951.webp	Swedish Massage	\N	Liệu pháp massage nhẹ nhàng toàn thân giúp thư giãn, tăng tuần hoàn máu và giảm căng cơ.	\N	A gentle, relaxing full-body massage that improves circulation, reduces stress, and eases muscle tension.	\N	300000	t	2025-05-11 07:51:16.172461+00	2025-05-11 08:38:23.666665+00
7	2	Chăm sóc da mặt cơ bản	uploads/services/1-4ffba486-f42c-495b-a3e1-6f56068c9067.webp	Basic Facial	\N	Liệu trình làm sạch, tẩy tế bào chết và dưỡng ẩm giúp làn da sáng khỏe, mịn màng.	\N	A gentle skincare treatment that cleanses, exfoliates, and moisturizes the skin for a fresh, healthy glow.	\N	250000	t	2025-05-11 07:55:12.935828+00	2025-05-11 08:38:45.631278+00
8	2	Chăm sóc da mặt chuyên sâu	uploads/services/1-84104fde-8e8b-4a95-84a4-2b98983c72af.webp	Deep Cleansing Facial	\N	Liệu trình chăm sóc da chuyên sâu giúp làm sạch sâu, tẩy tế bào chết, hút mụn và nuôi dưỡng làn da khỏe mạnh, sáng mịn.	\N	An intensive skincare treatment that deeply cleanses, exfoliates, extracts impurities, and nourishes the skin for a clear and radiant complexion.	\N	450000	t	2025-05-11 07:57:46.166727+00	2025-05-11 08:38:56.893044+00
9	4	Gội đầu dưỡng sinh	uploads/services/1-e94fbbc9-22ee-409c-bbd4-fbbcf018f5b2.webp	Traditional Scalp Massage1	\N	Liệu pháp gội đầu thư giãn, kích thích tuần hoàn máu, nuôi dưỡng tóc từ chân, giúp tóc khỏe mạnh và da đầu thư giãn.	\N	A soothing treatment that promotes relaxation, improves blood circulation to the scalp, and nourishes hair follicles for healthier hair.	\N	150000	t	2025-05-11 07:59:28.281272+00	2025-05-11 08:41:27.737259+00
4	1	Massage body đá muối	uploads/services/1-3e7ac422-f934-4c20-a1e0-6e1e5dd70c07.webp	Salt stone body massage	\N	Liệu pháp massage toàn thân bằng đá muối ấm giúp thư giãn và thải độc cơ thể.	\N	A relaxing full-body massage using warm Himalayan salt stones to relieve tension and detoxify the body.	\N	350000	t	2025-05-11 07:44:13.30142+00	2025-05-11 08:36:04.114326+00
5	1	Massage cổ vai gáy	uploads/services/1-5baee80c-50fc-4b6b-b0b6-4ef23dd26666.webp	Neck & Shoulder Massage	\N	Giúp giảm căng cứng, đau mỏi vùng cổ, vai, gáy – phù hợp cho người thường xuyên stress hoặc ngồi lâu.	\N	Relieves tension and stiffness in the neck, shoulders, and upper back—ideal for stress and posture-related pain.	\N	280000	t	2025-05-11 07:46:50.430944+00	2025-05-11 08:37:24.3925+00
11	3	Triệt lông 1/2 chân	uploads/services/1-9433ecf4-f711-4824-bede-24c53d41c012.webp	Half-Leg Hair Removal	\N	Loại bỏ lông vùng nửa chân (trên hoặc dưới) giúp da mịn màng, sạch lông lâu dài.	\N	Effectively removes unwanted hair from the lower or upper legs, leaving skin smooth and long-lastingly hair-free.	\N	2400000	t	2025-05-11 08:43:21.631907+00	2025-05-11 08:43:21.631907+00
12	3	Triệt lông 1/2 tay	uploads/services/1-eb33b3a0-c3ba-47f2-b499-524da3b9fe31.webp	Half-Arm Hair Removal	\N	Loại bỏ lông vùng nửa cánh tay (trên hoặc dưới) giúp làn da sạch lông, mịn màng và lâu mọc lại.	\N	Removes unwanted hair from the upper or lower arms for smooth, hair-free skin that lasts.	\N	1600000	t	2025-05-11 08:44:42.934923+00	2025-05-11 08:44:42.934923+00
13	3	Triệt lông toàn thân	uploads/services/1-804d10c6-8695-477f-847a-76a31657e05b.webp	Full Body Hair Removal	\N	Liệu trình triệt lông toàn diện trên toàn cơ thể, mang lại làn da mịn màng và hiệu quả lâu dài.	\N	Comprehensive hair removal treatment covering all major body areas for smooth, long-lasting results.	\N	10500000	t	2025-05-11 08:47:06.129445+00	2025-05-11 08:47:06.129445+00
14	5	Làm móng tay	uploads/services/1-f23bf8d8-4de6-42c0-a56b-53da6868a13e.jpg	Manicure	\N	Chăm sóc móng tay gồm cắt dũa, làm sạch, chăm sóc da quanh móng và sơn (nếu có) giúp bàn tay đẹp và gọn gàng	\N	A nail care treatment that includes cleaning, shaping, cuticle care, and optional polish for healthy, beautiful hands.	\N	250000	t	2025-05-11 08:48:39.494461+00	2025-05-11 08:48:39.494461+00
15	5	Làm móng chân	uploads/services/1-419b689a-e935-460c-be0e-06536d41f3df.jpg	Pedicure	\N	Chăm sóc móng chân gồm ngâm, làm sạch, cắt dũa, tẩy da chết và sơn (nếu có), giúp đôi chân mềm mại và gọn gàng	\N	A foot and toenail care treatment that includes soaking, cleaning, shaping, exfoliation, and optional polish for soft, well-groomed feet.	\N	250000	t	2025-05-11 08:49:27.541713+00	2025-05-11 08:49:27.541713+00
\.


--
-- Data for Name: services; Type: TABLE DATA; Schema: users; Owner: postgres
--

COPY users.services (id, service_name, description, price, is_active, created_at, updated_at, image, service_type, service_name_en, service_name_ko, description_ko, description_en, is_category, has_child) FROM stdin;
1	Massage	Na Spa sẽ giúp bạn giải tỏa căng thẳng mệt mỏi, cải thiện tuần hoàn máu & giấc ngủ với liệu pháp thư giãn bằng mùi hương	0	t	2025-05-05 17:42:25.771746+00	2025-05-07 15:10:53.190935+00	uploads/services/1-c22199c3-cedc-4137-8782-973a8915703c.webp	\N	Massage	\N	\N	Na Spa will help you relieve stress and fatigue, improve blood circulation and sleep quality through aroma relaxation therapy.	\N	f
2	Chăm sóc da	Bao gồm các liệu trình: Sạch sâu cấp ẩm, Căng bóng da, Trị mụn với thời lượng từ 60 - 90 phút / liệu trình	0	t	2025-05-05 17:48:19.877074+00	2025-05-07 15:11:54.594433+00	uploads/services/1-28a00245-8be7-4af3-94fb-4ebd6beeb9db.webp	\N	Skincare	\N	\N	Deep cleansing and hydration, skin brightening, and acne treatment, each lasting 60 to 90 minutes per session.	\N	f
3	Triệt lông	Na spa sử dụng công nghệ Triệt lông Diode Laser hiện đại nhất và an toàn cho da, triệt tận gốc lông và không sưng, đau.	0	t	2025-05-05 17:49:16.589262+00	2025-05-07 15:13:22.510422+00	uploads/services/1-b8cb22ac-a711-4ac4-a3f8-ed6f4151767b.webp	\N	Hair Removal	\N	\N	Na spa use the latest and safest Diode Laser hair removal technology, which eliminates hair at the root without causing swelling or pain.	\N	f
5	Nail	Dịch vụ chăm sóc và trang trí móng tay, móng chân giúp tôn lên vẻ đẹp và thể hiện cá tính.	0	t	2025-05-05 18:42:52.927206+00	2025-05-07 15:15:09.010107+00	uploads/services/1-27827a75-a19c-434e-8d86-19cb553a9f76.jpg	\N	Nail	\N	\N	Nail care and decoration services that enhance your beauty and express your unique personality.	\N	f
4	Gội đầu dưỡng sinh	Sở hữu da đầu khỏe mạnh và mái tóc óng mượt. Cải thiện sức khỏe, giảm lo âu, mệt mỏi	0	t	2025-05-05 17:51:01.876655+00	2025-05-07 17:05:55.896238+00	uploads/services/1-b0372090-9cc8-4b28-9bde-a191d35d5038.webp	\N	Scalp Therapy	\N	\N	Achieve a healthy scalp and shiny, smooth hair. Improve your overall well-being while reducing stress and fatigue.	\N	f
\.


--
-- Data for Name: tbl_users; Type: TABLE DATA; Schema: users; Owner: postgres
--

COPY users.tbl_users (pk_user_id, user_name, phone, full_name, role, password_hash, email_address, created_at, updated_at, is_active, is_verify, date_of_birth, address, avatar) FROM stdin;
1	admin	\N	tesst1	ADMIN	$argon2id$v=19$m=19456,t=2,p=1$zsPeAXXDEVtw4/k9WBra9g$ON0/amj0Gceh7oHa0VgGHomWEB163WemGRCLG8mRElw		2025-04-12 04:29:23.337569+00	2025-05-05 10:29:17.727767+00	t	f	06/08/1997		uploads/avatars/1-0466931f-8e78-4c32-a4c6-cf76fb8d99f6.png
9	\N	+84961483800	Trang Nhung	CUSTOMER	$argon2id$v=19$m=19456,t=2,p=1$JzKBSKCxoCCMlH5LE6Vt3g$YiLA7nNMkcwAlGx60HTCt7OAY7hl7wkc0aK4pF3Mo2U	test@gmail.com	2025-04-12 18:23:53.993621+00	2025-05-15 14:42:46.826182+00	t	t	06/08/1997	hậu thành	uploads/avatars/9-653d0721-0f80-494f-a100-3ea49b1814b5.jpg
8	\N	+84961483678	\N	CUSTOMER	\N	\N	2025-04-12 18:22:45.742613+00	2025-04-12 18:22:45.742613+00	t	f	\N	\N	\N
12	\N	+84961483890	\N	CUSTOMER	\N	\N	2025-04-24 23:15:18.686641+00	2025-04-24 23:15:18.686641+00	t	f	\N	\N	\N
13	\N	+84961483900	\N	CUSTOMER	\N	\N	2025-04-24 23:38:08.336673+00	2025-04-24 23:38:08.336673+00	t	f	\N	\N	\N
\.


--
-- Name: appointments_id_seq; Type: SEQUENCE SET; Schema: users; Owner: postgres
--

SELECT pg_catalog.setval('users.appointments_id_seq', 11, true);


--
-- Name: appointments_services_id_seq; Type: SEQUENCE SET; Schema: users; Owner: postgres
--

SELECT pg_catalog.setval('users.appointments_services_id_seq', 15, true);


--
-- Name: chat_messages_id_seq; Type: SEQUENCE SET; Schema: users; Owner: postgres
--

SELECT pg_catalog.setval('users.chat_messages_id_seq', 1, false);


--
-- Name: phone_codes_id_seq; Type: SEQUENCE SET; Schema: users; Owner: postgres
--

SELECT pg_catalog.setval('users.phone_codes_id_seq', 61, true);


--
-- Name: refresh_tokens_id_seq; Type: SEQUENCE SET; Schema: users; Owner: postgres
--

SELECT pg_catalog.setval('users.refresh_tokens_id_seq', 62, true);


--
-- Name: service_items_id_seq; Type: SEQUENCE SET; Schema: users; Owner: postgres
--

SELECT pg_catalog.setval('users.service_items_id_seq', 15, true);


--
-- Name: services_id_seq; Type: SEQUENCE SET; Schema: users; Owner: postgres
--

SELECT pg_catalog.setval('users.services_id_seq', 12, true);


--
-- Name: tbl_users_pk_user_id_seq; Type: SEQUENCE SET; Schema: users; Owner: postgres
--

SELECT pg_catalog.setval('users.tbl_users_pk_user_id_seq', 13, true);


--
-- Name: _sqlx_migrations _sqlx_migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public._sqlx_migrations
    ADD CONSTRAINT _sqlx_migrations_pkey PRIMARY KEY (version);


--
-- Name: appointments appointments_pkey; Type: CONSTRAINT; Schema: users; Owner: postgres
--

ALTER TABLE ONLY users.appointments
    ADD CONSTRAINT appointments_pkey PRIMARY KEY (id);


--
-- Name: appointments_services appointments_services_pkey; Type: CONSTRAINT; Schema: users; Owner: postgres
--

ALTER TABLE ONLY users.appointments_services
    ADD CONSTRAINT appointments_services_pkey PRIMARY KEY (id);


--
-- Name: chat_messages chat_messages_pkey; Type: CONSTRAINT; Schema: users; Owner: postgres
--

ALTER TABLE ONLY users.chat_messages
    ADD CONSTRAINT chat_messages_pkey PRIMARY KEY (id);


--
-- Name: phone_codes phone_codes_pkey; Type: CONSTRAINT; Schema: users; Owner: postgres
--

ALTER TABLE ONLY users.phone_codes
    ADD CONSTRAINT phone_codes_pkey PRIMARY KEY (id);


--
-- Name: refresh_tokens refresh_tokens_pkey; Type: CONSTRAINT; Schema: users; Owner: postgres
--

ALTER TABLE ONLY users.refresh_tokens
    ADD CONSTRAINT refresh_tokens_pkey PRIMARY KEY (id);


--
-- Name: refresh_tokens refresh_tokens_token_key; Type: CONSTRAINT; Schema: users; Owner: postgres
--

ALTER TABLE ONLY users.refresh_tokens
    ADD CONSTRAINT refresh_tokens_token_key UNIQUE (token);


--
-- Name: service_items service_items_pkey; Type: CONSTRAINT; Schema: users; Owner: postgres
--

ALTER TABLE ONLY users.service_items
    ADD CONSTRAINT service_items_pkey PRIMARY KEY (id);


--
-- Name: services services_pkey; Type: CONSTRAINT; Schema: users; Owner: postgres
--

ALTER TABLE ONLY users.services
    ADD CONSTRAINT services_pkey PRIMARY KEY (id);


--
-- Name: tbl_users tbl_users_email_address_key; Type: CONSTRAINT; Schema: users; Owner: postgres
--

ALTER TABLE ONLY users.tbl_users
    ADD CONSTRAINT tbl_users_email_address_key UNIQUE (email_address);


--
-- Name: tbl_users tbl_users_phone_key; Type: CONSTRAINT; Schema: users; Owner: postgres
--

ALTER TABLE ONLY users.tbl_users
    ADD CONSTRAINT tbl_users_phone_key UNIQUE (phone);


--
-- Name: tbl_users tbl_users_pkey; Type: CONSTRAINT; Schema: users; Owner: postgres
--

ALTER TABLE ONLY users.tbl_users
    ADD CONSTRAINT tbl_users_pkey PRIMARY KEY (pk_user_id);


--
-- Name: tbl_users tbl_users_user_name_key; Type: CONSTRAINT; Schema: users; Owner: postgres
--

ALTER TABLE ONLY users.tbl_users
    ADD CONSTRAINT tbl_users_user_name_key UNIQUE (user_name);


--
-- Name: idx_appointments_services_appointment_id; Type: INDEX; Schema: users; Owner: postgres
--

CREATE INDEX idx_appointments_services_appointment_id ON users.appointments_services USING btree (appointment_id);


--
-- Name: idx_appointments_services_service_id; Type: INDEX; Schema: users; Owner: postgres
--

CREATE INDEX idx_appointments_services_service_id ON users.appointments_services USING btree (service_id);


--
-- Name: idx_appointments_services_technician_id; Type: INDEX; Schema: users; Owner: postgres
--

CREATE INDEX idx_appointments_services_technician_id ON users.appointments_services USING btree (technician_id);


--
-- Name: idx_appointments_start_time; Type: INDEX; Schema: users; Owner: postgres
--

CREATE INDEX idx_appointments_start_time ON users.appointments USING btree (start_time);


--
-- Name: idx_appointments_status; Type: INDEX; Schema: users; Owner: postgres
--

CREATE INDEX idx_appointments_status ON users.appointments USING btree (status);


--
-- Name: idx_appointments_user_id; Type: INDEX; Schema: users; Owner: postgres
--

CREATE INDEX idx_appointments_user_id ON users.appointments USING btree (user_id);


--
-- Name: idx_chat_messages_receiver_id; Type: INDEX; Schema: users; Owner: postgres
--

CREATE INDEX idx_chat_messages_receiver_id ON users.chat_messages USING btree (receiver_id);


--
-- Name: idx_chat_messages_sender_id; Type: INDEX; Schema: users; Owner: postgres
--

CREATE INDEX idx_chat_messages_sender_id ON users.chat_messages USING btree (sender_id);


--
-- Name: idx_phone_codes_user_id; Type: INDEX; Schema: users; Owner: postgres
--

CREATE INDEX idx_phone_codes_user_id ON users.phone_codes USING btree (user_id);


--
-- Name: idx_refresh_tokens_token; Type: INDEX; Schema: users; Owner: postgres
--

CREATE INDEX idx_refresh_tokens_token ON users.refresh_tokens USING btree (token);


--
-- Name: idx_refresh_tokens_user_id; Type: INDEX; Schema: users; Owner: postgres
--

CREATE INDEX idx_refresh_tokens_user_id ON users.refresh_tokens USING btree (user_id);


--
-- Name: idx_service_items_parent_service_id; Type: INDEX; Schema: users; Owner: postgres
--

CREATE INDEX idx_service_items_parent_service_id ON users.service_items USING btree (parent_service_id);


--
-- Name: idx_service_items_service_name; Type: INDEX; Schema: users; Owner: postgres
--

CREATE INDEX idx_service_items_service_name ON users.service_items USING btree (service_name);


--
-- Name: idx_services_service_name; Type: INDEX; Schema: users; Owner: postgres
--

CREATE INDEX idx_services_service_name ON users.services USING btree (service_name);


--
-- Name: appointments_services update_appointments_services_timestamp; Type: TRIGGER; Schema: users; Owner: postgres
--

CREATE TRIGGER update_appointments_services_timestamp BEFORE UPDATE ON users.appointments_services FOR EACH ROW EXECUTE FUNCTION users.update_timestamp();


--
-- Name: appointments update_appointments_timestamp; Type: TRIGGER; Schema: users; Owner: postgres
--

CREATE TRIGGER update_appointments_timestamp BEFORE UPDATE ON users.appointments FOR EACH ROW EXECUTE FUNCTION users.update_timestamp();


--
-- Name: phone_codes update_phone_code_timestamp; Type: TRIGGER; Schema: users; Owner: postgres
--

CREATE TRIGGER update_phone_code_timestamp BEFORE UPDATE ON users.phone_codes FOR EACH ROW EXECUTE FUNCTION users.update_timestamp();


--
-- Name: refresh_tokens update_refresh_tokens_timestamp; Type: TRIGGER; Schema: users; Owner: postgres
--

CREATE TRIGGER update_refresh_tokens_timestamp BEFORE UPDATE ON users.refresh_tokens FOR EACH ROW EXECUTE FUNCTION users.update_timestamp();


--
-- Name: service_items update_service_items_timestamp; Type: TRIGGER; Schema: users; Owner: postgres
--

CREATE TRIGGER update_service_items_timestamp BEFORE UPDATE ON users.service_items FOR EACH ROW EXECUTE FUNCTION users.update_timestamp();


--
-- Name: services update_services_timestamp; Type: TRIGGER; Schema: users; Owner: postgres
--

CREATE TRIGGER update_services_timestamp BEFORE UPDATE ON users.services FOR EACH ROW EXECUTE FUNCTION users.update_timestamp();


--
-- Name: tbl_users update_user_timestamp; Type: TRIGGER; Schema: users; Owner: postgres
--

CREATE TRIGGER update_user_timestamp BEFORE UPDATE ON users.tbl_users FOR EACH ROW EXECUTE FUNCTION users.update_timestamp();


--
-- Name: appointments appointments_receptionist_id_fkey; Type: FK CONSTRAINT; Schema: users; Owner: postgres
--

ALTER TABLE ONLY users.appointments
    ADD CONSTRAINT appointments_receptionist_id_fkey FOREIGN KEY (receptionist_id) REFERENCES users.tbl_users(pk_user_id) ON DELETE SET NULL;


--
-- Name: appointments_services appointments_services_appointment_id_fkey; Type: FK CONSTRAINT; Schema: users; Owner: postgres
--

ALTER TABLE ONLY users.appointments_services
    ADD CONSTRAINT appointments_services_appointment_id_fkey FOREIGN KEY (appointment_id) REFERENCES users.appointments(id) ON DELETE CASCADE;


--
-- Name: appointments_services appointments_services_service_id_fkey; Type: FK CONSTRAINT; Schema: users; Owner: postgres
--

ALTER TABLE ONLY users.appointments_services
    ADD CONSTRAINT appointments_services_service_id_fkey FOREIGN KEY (service_id) REFERENCES users.service_items(id) ON DELETE RESTRICT;


--
-- Name: appointments_services appointments_services_technician_id_fkey; Type: FK CONSTRAINT; Schema: users; Owner: postgres
--

ALTER TABLE ONLY users.appointments_services
    ADD CONSTRAINT appointments_services_technician_id_fkey FOREIGN KEY (technician_id) REFERENCES users.tbl_users(pk_user_id) ON DELETE SET NULL;


--
-- Name: appointments_services appointments_services_updated_by_fkey; Type: FK CONSTRAINT; Schema: users; Owner: postgres
--

ALTER TABLE ONLY users.appointments_services
    ADD CONSTRAINT appointments_services_updated_by_fkey FOREIGN KEY (updated_by) REFERENCES users.tbl_users(pk_user_id) ON DELETE SET NULL;


--
-- Name: appointments appointments_technician_id_fkey; Type: FK CONSTRAINT; Schema: users; Owner: postgres
--

ALTER TABLE ONLY users.appointments
    ADD CONSTRAINT appointments_technician_id_fkey FOREIGN KEY (technician_id) REFERENCES users.tbl_users(pk_user_id) ON DELETE SET NULL;


--
-- Name: appointments appointments_updated_by_fkey; Type: FK CONSTRAINT; Schema: users; Owner: postgres
--

ALTER TABLE ONLY users.appointments
    ADD CONSTRAINT appointments_updated_by_fkey FOREIGN KEY (updated_by) REFERENCES users.tbl_users(pk_user_id) ON DELETE SET NULL;


--
-- Name: appointments appointments_user_id_fkey; Type: FK CONSTRAINT; Schema: users; Owner: postgres
--

ALTER TABLE ONLY users.appointments
    ADD CONSTRAINT appointments_user_id_fkey FOREIGN KEY (user_id) REFERENCES users.tbl_users(pk_user_id) ON DELETE RESTRICT;


--
-- Name: chat_messages chat_messages_receiver_id_fkey; Type: FK CONSTRAINT; Schema: users; Owner: postgres
--

ALTER TABLE ONLY users.chat_messages
    ADD CONSTRAINT chat_messages_receiver_id_fkey FOREIGN KEY (receiver_id) REFERENCES users.tbl_users(pk_user_id) ON DELETE CASCADE;


--
-- Name: chat_messages chat_messages_sender_id_fkey; Type: FK CONSTRAINT; Schema: users; Owner: postgres
--

ALTER TABLE ONLY users.chat_messages
    ADD CONSTRAINT chat_messages_sender_id_fkey FOREIGN KEY (sender_id) REFERENCES users.tbl_users(pk_user_id) ON DELETE CASCADE;


--
-- Name: phone_codes phone_codes_user_id_fkey; Type: FK CONSTRAINT; Schema: users; Owner: postgres
--

ALTER TABLE ONLY users.phone_codes
    ADD CONSTRAINT phone_codes_user_id_fkey FOREIGN KEY (user_id) REFERENCES users.tbl_users(pk_user_id) ON DELETE CASCADE;


--
-- Name: refresh_tokens refresh_tokens_user_id_fkey; Type: FK CONSTRAINT; Schema: users; Owner: postgres
--

ALTER TABLE ONLY users.refresh_tokens
    ADD CONSTRAINT refresh_tokens_user_id_fkey FOREIGN KEY (user_id) REFERENCES users.tbl_users(pk_user_id) ON DELETE CASCADE;


--
-- Name: service_items service_items_parent_service_id_fkey; Type: FK CONSTRAINT; Schema: users; Owner: postgres
--

ALTER TABLE ONLY users.service_items
    ADD CONSTRAINT service_items_parent_service_id_fkey FOREIGN KEY (parent_service_id) REFERENCES users.services(id) ON DELETE CASCADE;


--
-- PostgreSQL database dump complete
--

--
-- PostgreSQL database cluster dump complete
--

