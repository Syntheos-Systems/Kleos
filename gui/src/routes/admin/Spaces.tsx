import { useQuery, useQueryClient } from '@tanstack/react-query';
import { useMemo, useState } from 'react';
import type { FormEvent } from 'react';
import {
  createInstanceGrant,
  getMe,
  listInstanceGrants,
  listUsers,
  revokeInstanceGrant
} from '$lib/api/admin';
import type { InstanceAccess } from '$lib/types';
import { Badge } from '../../ui/Badge';
import { EmptyState } from '../../ui/EmptyState';
import { Panel } from '../../ui/Panel';
import { Spinner } from '../../ui/Spinner';
import { Table } from '../../ui/Table';
import './admin.css';

// Render the admin Spaces and Sharing page: grant or revoke delegated access to
// a user's entire instance (whole-instance, sharded sharing model).
export function Spaces() {
  const queryClient = useQueryClient();
  const me = useQuery({ queryFn: getMe, queryKey: ['me'] });
  const isAdmin = me.data?.is_admin === true;

  const users = useQuery({ enabled: isAdmin, queryFn: listUsers, queryKey: ['admin', 'users'] });

  const [ownerId, setOwnerId] = useState<number | null>(null);
  const [granteeId, setGranteeId] = useState<number | null>(null);
  const [access, setAccess] = useState<InstanceAccess>('read');
  const [error, setError] = useState<string | null>(null);

  const grantsKey = useMemo(() => ['admin', 'instance-grants', ownerId] as const, [ownerId]);
  const grants = useQuery({
    enabled: ownerId != null,
    queryFn: () => listInstanceGrants(ownerId as number),
    queryKey: grantsKey
  });

  // Resolve a username for display, falling back to the numeric id.
  const userName = (id: number) => users.data?.find((u) => u.id === id)?.username ?? `#${id}`;

  // Create the grant described by the form, then refresh the grants table.
  async function handleGrant(event: FormEvent) {
    event.preventDefault();
    setError(null);
    if (ownerId == null || granteeId == null) {
      return;
    }
    try {
      await createInstanceGrant({ access, grantee_user_id: granteeId, owner_user_id: ownerId });
      setGranteeId(null);
      await queryClient.invalidateQueries({ queryKey: grantsKey });
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : 'Grant failed');
    }
  }

  // Revoke one grantee's access to the selected owner's instance.
  async function handleRevoke(granteeUserId: number) {
    setError(null);
    try {
      await revokeInstanceGrant(ownerId as number, granteeUserId);
      await queryClient.invalidateQueries({ queryKey: grantsKey });
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : 'Revoke failed');
    }
  }

  if (me.isLoading) {
    return <Spinner />;
  }

  if (!isAdmin) {
    return (
      <div>
        <header className="route-header">
          <div>
            <h1>Spaces &amp; Sharing</h1>
            <p>Delegated instance access</p>
          </div>
        </header>
        <Panel>
          <EmptyState
            message="You need the admin scope to manage instance access grants."
            title="Admins only"
          />
        </Panel>
      </div>
    );
  }

  const userList = users.data ?? [];
  // A user cannot be granted access to their own instance, so exclude the owner.
  const granteeOptions = userList.filter((u) => u.id !== ownerId);

  return (
    <div>
      <header className="route-header">
        <div>
          <h1>Spaces &amp; Sharing</h1>
          <p>Grant delegated read or write access to a user&apos;s entire instance</p>
        </div>
      </header>

      <Panel title="Instance owner">
        <label className="admin-field">
          <span>Manage access to</span>
          <select
            onChange={(event) => {
              setOwnerId(event.target.value ? Number(event.target.value) : null);
              setGranteeId(null);
            }}
            value={ownerId ?? ''}
          >
            <option value="">Select a user...</option>
            {userList.map((u) => (
              <option key={u.id} value={u.id}>
                {u.username}
              </option>
            ))}
          </select>
        </label>
      </Panel>

      {ownerId != null ? (
        <div className="admin-stack">
          <Panel title="Grant access">
            <form className="admin-grant-form" onSubmit={handleGrant}>
              <label className="admin-field">
                <span>Grantee</span>
                <select
                  onChange={(event) => setGranteeId(event.target.value ? Number(event.target.value) : null)}
                  value={granteeId ?? ''}
                >
                  <option value="">Select a user...</option>
                  {granteeOptions.map((u) => (
                    <option key={u.id} value={u.id}>
                      {u.username}
                    </option>
                  ))}
                </select>
              </label>
              <label className="admin-field">
                <span>Access</span>
                <select onChange={(event) => setAccess(event.target.value as InstanceAccess)} value={access}>
                  <option value="read">Read</option>
                  <option value="write">Write</option>
                </select>
              </label>
              <button className="admin-grant-button" disabled={granteeId == null} type="submit">
                Grant
              </button>
            </form>
            {error ? (
              <p className="admin-error" role="alert">
                {error}
              </p>
            ) : null}
          </Panel>

          <Panel title={`Access to ${userName(ownerId)}'s instance`}>
            {grants.isLoading ? (
              <Spinner />
            ) : grants.data && grants.data.length > 0 ? (
              <Table
                headers={['Grantee', 'Access', 'Granted by', 'Created', '']}
                rows={grants.data.map((g) => [
                  userName(g.grantee_user_id),
                  <Badge key="a" label={g.access} tone={g.access === 'write' ? 'warn' : 'ok'} />,
                  userName(g.granted_by),
                  g.created_at ? g.created_at.slice(0, 19) : '--',
                  <button
                    className="admin-revoke"
                    key="r"
                    onClick={() => handleRevoke(g.grantee_user_id)}
                    type="button"
                  >
                    Revoke
                  </button>
                ])}
              />
            ) : (
              <EmptyState
                hint="Use the form above to grant read or write access."
                message="No one has delegated access to this instance yet."
                title="No grants"
              />
            )}
          </Panel>
        </div>
      ) : null}
    </div>
  );
}
