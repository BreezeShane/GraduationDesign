import React, { useEffect, useState } from 'react';
import { Button, Divider, Form, Input, Radio, Space, Switch, Table, Tag, Transfer } from 'antd';
import { UserOutlined, KeyOutlined, MailOutlined } from '@ant-design/icons';
import type { FormProps, GetProp, RadioChangeEvent, TableColumnsType, TableProps, TransferProps } from 'antd';
import { UnlockOutlined, LockOutlined } from "@ant-design/icons"
import { NotificationInstance } from 'antd/es/notification/interface';
import axios from 'axios';

type TransferItem = GetProp<TransferProps, 'dataSource'>[number];
type TableRowSelection<T extends object> = TableProps<T>['rowSelection'];

interface FieldType {
  username: string;
  email: string;
  password: string;
  repassword: string;
  role: string;
};

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
  const [role, setRole] = useState("");
  const [form] = Form.useForm();

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
  }, []);

  const clearForm = () => {
    form.resetFields();
  }

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

  const onRadioChange = (e: RadioChangeEvent) => {
    setRole(e.target.value);
  }

  const onFinish: FormProps<FieldType>['onFinish'] = (values) => {
    const useremail = sessionStorage.getItem('useremail');
    if (!useremail) {
      messageClient.success({
        message: `Failed to operate!`,
        description: "Please check your status of signing in!",
        placement: 'topLeft',
        duration: 2,
      });
      return;
    }
    axios.post('/admin/user_manage/add_admin', {
      admin_email: useremail,
      username: values.username,
      useremail: values.email,
      password: values.password,
      repassword: values.repassword,
      role: values.role,
    }).then(function (response) {
      messageClient.success({
        message: `Success to sign up a new admin!`,
        description: "Now you can go to sign in by this administrator!",
        placement: 'topLeft',
        duration: 2,
      });
    })
    .catch(function (error) {
        console.log(error);
        messageClient.success({
          message: `Failed to sign up a new account!`,
          description: "Please check your inputs!",
          placement: 'topLeft',
          duration: 2,
        });
    });
  };

  const onFinishFailed: FormProps<FieldType>['onFinishFailed'] = (errorInfo) => {
    console.log('Failed:', errorInfo);
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

      <Divider>Add Administrator</Divider>

      <div style={{ display: "flex", alignItems: "center", justifyContent: "center" }}>
        <Form
          name="add_admin"
          form={form}
          labelCol={{ span: 8 }}
          wrapperCol={{ span: 30 }}
          style={{ maxWidth: 600 }}
          onFinish={onFinish}
          onFinishFailed={onFinishFailed}
          autoComplete="off"
        >
          <Form.Item<FieldType>
            label="User Name"
            name="username"
            rules={[{ required: true, message: 'Please input your username!' }]}
          >
            <Input size="large" placeholder="User Name" prefix={<UserOutlined />} />
          </Form.Item>

          <Form.Item<FieldType>
            label="User Email"
            name="email"
            rules={[{ required: true, message: 'Please input your Email!' }]}
          >
            <Input size="large" placeholder="User EMail" prefix={<MailOutlined />} />
          </Form.Item>
          <Form.Item<FieldType>
              label="Password"
              name="password"
              rules={[{
                required: true,
                pattern:
                    /^(?![^a-zA-Z]+$)(?!\\D+$).{8,16}$/,
                message: "Need 8-16 characters containing letters and numbers",
            }]}
          >
            <Input.Password size="large" placeholder="Password" prefix={<KeyOutlined />} />
          </Form.Item>

          <Form.Item<FieldType>
            label="Confirm Password"
            name="repassword"
            dependencies={['password']}
            rules={[
              ({ getFieldValue }) => ({
                  validator(rule, value) {
                      if (!value || getFieldValue('password') === value) {
                          return Promise.resolve();
                      }
                      return Promise.reject('Please keep the same to your password above!');
                  },
              }),
              {
                required: true
              }
          ]}
          >
            <Input.Password size="large" placeholder="Confirm Password" prefix={<KeyOutlined />} />
          </Form.Item>

          <Form.Item<FieldType>
            label="Role"
            name="role"
            rules={[{ required: true, message: 'Please select the user role!' }]}
          >
            <Radio.Group onChange={onRadioChange} value={role}>
              <Space direction="vertical">
                <Radio value={"User Administrator"}>User Admin</Radio>
                <Radio value={"Model Administrator"}>Model Admin</Radio>
              </Space>
            </Radio.Group>
          </Form.Item>

        <Form.Item wrapperCol={{ offset: 8, span: 16 }}>
            <Space>
              <Button type="primary" htmlType="submit">
                Submit
              </Button>
              <Button danger onClick={clearForm}>
                Clear
              </Button>
            </Space>
          </Form.Item>
      </Form>
    </div>
  </>
);};

export default UserManage;