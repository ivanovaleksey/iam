do
$$
declare
  iam_account_id     uuid;
  foxford_account_id uuid;

  iam_ns_id          uuid;
  foxford_ns_id      uuid;
begin
  delete from account;

  insert into account (enabled) values (true) returning id into iam_account_id;
  insert into namespace (label, account_id, enabled) values ('iam.ng.services', iam_account_id, true) returning id into iam_ns_id;

  insert into abac_subject_attr (namespace_id, subject_id, key, value) values (iam_ns_id, iam_account_id, 'role', 'admin');
  insert into abac_subject_attr (namespace_id, subject_id, key, value) values (iam_ns_id, iam_account_id, 'role', 'client');
  insert into abac_subject_attr (namespace_id, subject_id, key, value) values (iam_ns_id, iam_account_id, 'owner:namespace', iam_ns_id);

  insert into abac_object_attr (namespace_id, object_id, key, value) values (iam_ns_id, 'namespace', 'type', 'namespace');
  insert into abac_object_attr (namespace_id, object_id, key, value) values (iam_ns_id, 'namespace.' || iam_ns_id, 'type', 'namespace');
  insert into abac_object_attr (namespace_id, object_id, key, value) values (iam_ns_id, 'namespace.' || iam_ns_id, 'belongs_to:namespace', iam_ns_id);

  insert into abac_action_attr (namespace_id, action_id, key, value) values (iam_ns_id, 'create', 'action', 'create');
  insert into abac_action_attr (namespace_id, action_id, key, value) values (iam_ns_id, 'read', 'action', 'read');
  insert into abac_action_attr (namespace_id, action_id, key, value) values (iam_ns_id, 'update', 'action', 'update');
  insert into abac_action_attr (namespace_id, action_id, key, value) values (iam_ns_id, 'delete', 'action', 'delete');
  insert into abac_action_attr (namespace_id, action_id, key, value) values (iam_ns_id, 'list', 'action', 'list');
  insert into abac_action_attr (namespace_id, action_id, key, value) values (iam_ns_id, 'execute', 'action', 'execute');

  insert into abac_action_attr (namespace_id, action_id, key, value) values (iam_ns_id, 'create', 'action', '*');
  insert into abac_action_attr (namespace_id, action_id, key, value) values (iam_ns_id, 'read', 'action', '*');
  insert into abac_action_attr (namespace_id, action_id, key, value) values (iam_ns_id, 'update', 'action', '*');
  insert into abac_action_attr (namespace_id, action_id, key, value) values (iam_ns_id, 'delete', 'action', '*');
  insert into abac_action_attr (namespace_id, action_id, key, value) values (iam_ns_id, 'list', 'action', '*');
  insert into abac_action_attr (namespace_id, action_id, key, value) values (iam_ns_id, 'execute', 'action', '*');

  insert into abac_policy (namespace_id, subject_namespace_id, subject_key, subject_value, object_namespace_id, object_key, object_value, action_namespace_id, action_key, action_value)
  values (iam_ns_id, iam_ns_id, 'role', 'admin', iam_ns_id, 'type', 'namespace', iam_ns_id, 'action', '*');
end
$$ language plpgsql;
