```sql
do
$$
declare
    iam_account_id     uuid;
    foxford_account_id uuid;

    iam_ns_id          uuid;
    foxford_ns_id      uuid;

    -- account_id_        uuid;
begin
    delete from account;


    -- 1. Создаем аккаунт администратора
    insert into account (enabled) values (true) returning id into iam_account_id;
    insert into namespace (label, account_id, enabled) values ('iam.ng.services', iam_account_id, true) returning id into iam_ns_id;

    insert into abac_subject_attr (namespace_id, subject_id, key, value) values (iam_ns_id, iam_account_id, 'role', 'admin');
    insert into abac_subject_attr (namespace_id, subject_id, key, value) values (iam_ns_id, iam_account_id, 'owner:namespace', iam_ns_id);

--     TODO: Должна быть возможность создать это через API
--     insert into abac_subject_attr (namespace_id, subject_id, key, value) values (iam_ns_id, iam_account_id, 'role', 'client');

    insert into abac_object_attr (namespace_id, object_id, key, value) values (iam_ns_id, 'namespace', 'type', 'namespace');
    insert into abac_object_attr (namespace_id, object_id, key, value) values (iam_ns_id, 'namespace.' || iam_ns_id, 'type', 'namespace');
    insert into abac_object_attr (namespace_id, object_id, key, value) values (iam_ns_id, 'namespace.' || iam_ns_id, 'belongs_to:namespace', iam_ns_id);

    insert into abac_action_attr (namespace_id, action_id, key, value) values (iam_ns_id, 'create', 'action', 'crudl');
    insert into abac_action_attr (namespace_id, action_id, key, value) values (iam_ns_id, 'read', 'action', 'crudl');
    insert into abac_action_attr (namespace_id, action_id, key, value) values (iam_ns_id, 'update', 'action', 'crudl');
    insert into abac_action_attr (namespace_id, action_id, key, value) values (iam_ns_id, 'delete', 'action', 'crudl');
    insert into abac_action_attr (namespace_id, action_id, key, value) values (iam_ns_id, 'list', 'action', 'crudl');
    insert into abac_action_attr (namespace_id, action_id, key, value) values (iam_ns_id, 'execute', 'action', 'crudl');

    -- TODO: Аккаунт администратора должен обладать всеми правами.
    -- Должна быть возможность делать запросы на создание клиентов от лица аккаунта iam.
    insert into abac_policy (namespace_id, subject_namespace_id, subject_key, subject_value, object_namespace_id, object_key, object_value, action_namespace_id, action_key, action_value)
    values (iam_ns_id, iam_ns_id, 'role', 'admin', iam_ns_id, 'type', 'namespace', iam_ns_id, 'action', 'crudl');

    insert into abac_policy (namespace_id, subject_namespace_id, subject_key, subject_value, object_namespace_id, object_key, object_value, action_namespace_id, action_key, action_value)
    values (iam_ns_id, iam_ns_id, 'role', 'admin', iam_ns_id, 'type', 'abac_subject_attr', iam_ns_id, 'action', 'crudl');

    -- ...


--   2. Создаем клиентский аккаунт foxford

-- Use-case: пользователь фоксфорда "создает себя" через endpoint аутентификации.
--     Потом этому пользователю назначается роль `client`. Пользователь должен получить все права.

    -- 2.1 Создаем identity -> создается account
    -- POST identity.create

    -- identity.create должен быть использован только для создания новых аккаунтов (первой identity)
    -- {
    --     "provider": "$iam_ns_id",
    --     "label": "trusted",
    --     "uid": "foxford.ru",
    --     "issuer_id": "$iam_account_id"
    -- }

    -- - Ищем identity по (provider, label, uid)
    -- - Создаем новый аккаунт, если identity не найдена
    -- - Создаем identity с этим аккаунтом


    -- Запрещено для пользователей
    -- auth.call(subject: iam_account_id, object: 'identity', action: 'create')
    insert into abac_policy (namespace_id, subject_namespace_id, subject_key, subject_value, object_namespace_id, object_key, object_value, action_namespace_id, action_key, action_value)
    values (iam_ns_id, iam_ns_id, 'role', 'client', iam_ns_id, 'type', 'identity', iam_ns_id, 'action', 'crudl');

    -- Для клиента возможно только создание идентичности провайдера, которым владеет клиент
    -- auth.call(subject: iam_account_id, object: 'namespace.iam_ns_id', action: 'execute')
    insert into abac_policy (namespace_id, subject_namespace_id, subject_key, subject_value, object_namespace_id, object_key, object_value, action_namespace_id, action_key, action_value)
    values (iam_ns_id, iam_ns_id, 'owner:namespace', iam_ns_id, iam_ns_id, 'belongs_to:namespace', iam_ns_id, iam_ns_id, 'action', 'crudl');


    insert into account (enabled) values (true) returning id into foxford_account_id;
    insert into identity (provider, label, uid, issuer_id, account_id) values (iam_ns_id, 'trusted', 'foxford.ru', iam_account_id, foxford_account_id);


    -- 2.2 Создаем namespace
    -- POST namespace.create

    -- {
    --     "label": "foxford.ru",
    --     "account_id": "$foxford_account_id",
    --     "enabled": true
    -- }

    -- auth.call(subject: iam_account_id, object: 'namespace', action: 'create')
    insert into namespace (label, account_id, enabled) values ('foxford.ru', foxford_account_id, true) returning id into foxford_ns_id;
end
$$ language plpgsql;
```

```sql
with iam_ns as (
    select *
    from namespace
    where label = 'iam.ng.services'
    limit 1
)

select *
from abac_policy
  join abac_subject_attr on abac_policy.subject_namespace_id = abac_subject_attr.namespace_id
                            and abac_policy.subject_key = abac_subject_attr.key
                            and abac_policy.subject_value = abac_subject_attr.value
  join abac_object_attr on abac_policy.object_namespace_id = abac_object_attr.namespace_id
                           and abac_policy.object_key = abac_object_attr.key
                           and abac_policy.object_value = abac_object_attr.value
  join abac_action_attr on abac_policy.action_namespace_id = abac_action_attr.namespace_id
                           and abac_policy.action_key = abac_action_attr.key
                           and abac_policy.action_value = abac_action_attr.value
where abac_policy.namespace_id = (select id from iam_ns)
      and abac_subject_attr.subject_id = (select account_id from iam_ns)
      and abac_object_attr.object_id = (select 'namespace.' || id from iam_ns)
      and abac_action_attr.action_id = 'execute';
```

```sql
with iam_ns as (
    select *
    from namespace
    where label = 'iam.ng.services'
    limit 1
)

select *
from abac_policy
  join abac_subject_attr on abac_policy.subject_namespace_id = abac_subject_attr.namespace_id
                            and abac_policy.subject_key = abac_subject_attr.key
                            and abac_policy.subject_value = abac_subject_attr.value
  join abac_object_attr on abac_policy.object_namespace_id = abac_object_attr.namespace_id
                           and abac_policy.object_key = abac_object_attr.key
                           and abac_policy.object_value = abac_object_attr.value
  join abac_action_attr on abac_policy.action_namespace_id = abac_action_attr.namespace_id
                           and abac_policy.action_key = abac_action_attr.key
                           and abac_policy.action_value = abac_action_attr.value
where abac_policy.namespace_id = (select id from iam_ns)
      and abac_subject_attr.subject_id = (select account_id from iam_ns)
      and abac_object_attr.object_id = 'namespace'
      and abac_action_attr.action_id = 'create';
```
