do
$$
declare
  _a_iam_id uuid;
  _n_iam_id uuid := 'bab37008-3dc5-492c-af73-80c241241d71';
begin
  insert into account default values returning id into _a_iam_id;

  insert into refresh_token (account_id, algorithm, keys)
  values (_a_iam_id, 'HS256', array[gen_random_bytes(64)]);

  insert into namespace (id, label, account_id)
  values (_n_iam_id, 'iam.ng.services', _a_iam_id);

  insert into abac_subject (inbound, outbound)
  values (('account/' || _a_iam_id, 'uri', _n_iam_id) :: abac_attribute, ('admin', 'role', _n_iam_id) :: abac_attribute);

  insert into abac_object (inbound, outbound)
  values (('account', 'type', _n_iam_id) :: abac_attribute, ('namespace/' || _n_iam_id, 'uri', _n_iam_id) :: abac_attribute),
         (('identity', 'type', _n_iam_id) :: abac_attribute, ('namespace/' || _n_iam_id, 'uri', _n_iam_id) :: abac_attribute),
         (('namespace', 'type', _n_iam_id) :: abac_attribute, ('namespace/' || _n_iam_id, 'uri', _n_iam_id) :: abac_attribute),
         (('abac_subject', 'type', _n_iam_id) :: abac_attribute, ('namespace/' || _n_iam_id, 'uri', _n_iam_id) :: abac_attribute),
         (('abac_object', 'type', _n_iam_id) :: abac_attribute, ('namespace/' || _n_iam_id, 'uri', _n_iam_id) :: abac_attribute),
         (('abac_action', 'type', _n_iam_id) :: abac_attribute, ('namespace/' || _n_iam_id, 'uri', _n_iam_id) :: abac_attribute),
         (('abac_policy', 'type', _n_iam_id) :: abac_attribute, ('namespace/' || _n_iam_id, 'uri', _n_iam_id) :: abac_attribute);

  insert into abac_object (inbound, outbound)
  values (('account/' || _a_iam_id, 'uri', _n_iam_id) :: abac_attribute, ('account', 'type', _n_iam_id) :: abac_attribute);

  insert into abac_object (inbound, outbound)
  values (('namespace/' || _n_iam_id, 'uri', _n_iam_id) :: abac_attribute, ('account/' || _a_iam_id, 'uri', _n_iam_id) :: abac_attribute),
         (('namespace/' || _n_iam_id, 'uri', _n_iam_id) :: abac_attribute, ('namespace', 'type', _n_iam_id) :: abac_attribute);

  insert into abac_action (inbound, outbound)
  values (('create', 'operation', _n_iam_id) :: abac_attribute, ('any', 'operation', _n_iam_id) :: abac_attribute),
         (('read', 'operation', _n_iam_id) :: abac_attribute, ('any', 'operation', _n_iam_id) :: abac_attribute),
         (('update', 'operation', _n_iam_id) :: abac_attribute, ('any', 'operation', _n_iam_id) :: abac_attribute),
         (('delete', 'operation', _n_iam_id) :: abac_attribute, ('any', 'operation', _n_iam_id) :: abac_attribute),
         (('list', 'operation', _n_iam_id) :: abac_attribute, ('any', 'operation', _n_iam_id) :: abac_attribute);

  insert into abac_policy (subject, object, action, namespace_id)
  values (array[('account/' || _a_iam_id, 'uri', _n_iam_id) :: abac_attribute],
          array[('account/' || _a_iam_id, 'uri', _n_iam_id) :: abac_attribute],
          array[('any', 'operation', _n_iam_id) :: abac_attribute],
          _n_iam_id);
end
$$ language plpgsql;
