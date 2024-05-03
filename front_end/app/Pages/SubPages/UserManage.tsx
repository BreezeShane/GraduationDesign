import React, { useEffect, useState } from 'react';
import { Space, Switch, Table, Tag, Transfer } from 'antd';
import type { GetProp, TableColumnsType, TableProps, TransferProps } from 'antd';
import { UnlockOutlined, LockOutlined } from "@ant-design/icons"
import { NotificationInstance } from 'antd/es/notification/interface';
import axios from 'axios';

type TransferItem = GetProp<TransferProps, 'dataSource'>[number];
type TableRowSelection<T extends object> = TableProps<T>['rowSelection'];

interface UserType {
  username: string,
  useremail: string,
  user_identity: string,
  user_contribution: number,
  available: boolean
}

interface TableTransferProps extends TransferProps<TransferItem> {
  dataSource: UserType[];
  leftColumns: TableColumnsType<UserType>;
  rightColumns: TableColumnsType<UserType>;
}

// Customize Table Transfer
const TableTransfer = ({ leftColumns, rightColumns, ...restProps }: TableTransferProps) => (
  <Transfer {...restProps}>
    {({
      direction,
      filteredItems,
      onItemSelect,
      onItemSelectAll,
      selectedKeys: listSelectedKeys,
      disabled: listDisabled,
    }) => {
      const columns = direction === 'left' ? leftColumns : rightColumns;

      const rowSelection: TableRowSelection<TransferItem> = {
        getCheckboxProps: () => ({ disabled: listDisabled }),
        onChange(selectedRowKeys) {
          onItemSelectAll(selectedRowKeys, 'replace');
        },
        selectedRowKeys: listSelectedKeys,
        selections: [Table.SELECTION_ALL, Table.SELECTION_INVERT, Table.SELECTION_NONE],
      };

      return (
        <Table
          rowSelection={rowSelection}
          columns={columns}
          dataSource={filteredItems}
          size="small"
          style={{ pointerEvents: listDisabled ? 'none' : undefined }}
          onRow={({ key, disabled: itemDisabled }) => ({
            onClick: () => {
              if (itemDisabled || listDisabled) {
                return;
              }
              onItemSelect(key, !listSelectedKeys.includes(key));
            },
          })}
        />
      );
    }}
  </Transfer>
);

const columns: TableColumnsType<UserType> = [
  {
    title: 'User Name',
    dataIndex: 'username',
  },
  {
    title: 'User Email',
    dataIndex: 'useremail',
  },
  {
    title: 'User Identity',
    dataIndex: 'user_identity',
    render: (tag: string) => (
      <Tag style={{ marginInlineEnd: 0 }} color="cyan">
        {tag}
      </Tag>
    ),
  },
  {
    title: 'Contribution',
    dataIndex: 'user_contribution',
  },
];

const UserManage: React.FC<{ messageClient: NotificationInstance}> = (props) => {
  const { messageClient } = props;
  const [targetKeys, setTargetKeys] = useState<TransferProps['targetKeys']>([]);
  const [disabled, setDisabled] = useState(true);
  const [user_list, setUserList] = useState<UserType[]>([]);

  useEffect(() => {
    axios.get(`/admin/user_manage`, {
      params: {
        useremail: sessionStorage.getItem("useremail")
      }
    })
    .then((res) => {
      if (res.status === 200) {
        let users: UserType[] = [];
        let users_suspended: number[] = [];
        for (let idx in res.data) {
          let user = res.data[idx];
          user.key = idx;
          if (!user.available) {
            users_suspended.push(user.key);
          }
          users.push(user);
        }
        setUserList(users);
        setTargetKeys(users_suspended);
      }
    }).catch((err) => {
      console.log("get users error: ", err)
    });
  }, [])

  const onChange: TableTransferProps['onChange'] = (targetKeys, direction, moveKeys) => {
    setTargetKeys(targetKeys);
    let users: string[] = [];
    for (let target in moveKeys) {
      users.push(user_list[target].useremail);
    }
    axios.post("/admin/user_manage", {
      admin_email: sessionStorage.getItem('useremail'),
      user_emails: JSON.stringify(users),
    }).then((res) => {
      console.log(res);
    }).catch((err) => {
      console.log(err);
    });
  };

  const toggleDisabled = (checked: boolean) => {
    setDisabled(checked);
  };

  return (
    <>
        <Space style={{ marginTop: 16 }}>
            <Switch
                checkedChildren={<LockOutlined />}
                unCheckedChildren={<UnlockOutlined />}
                checked={disabled}
                onChange={toggleDisabled}
            />
        </Space>
        <TableTransfer
            dataSource={user_list}
            targetKeys={targetKeys}
            disabled={disabled}
            showSearch
            onChange={onChange}
            operations={['Suspend Accounts', 'Unsuspend Accounts']}
            filterOption={(inputValue, item) =>
                item.username!.indexOf(inputValue) !== -1 ||
                item.user_identity.indexOf(inputValue) !== -1 ||
                item.useremail.indexOf(inputValue) !== -1 ||
                item.user_contribution == parseInt(inputValue)
            }
            leftColumns={columns}
            rightColumns={columns}
        />
    </>
  );
};

export default UserManage;