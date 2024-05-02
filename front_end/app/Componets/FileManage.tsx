import React, { useEffect, useState } from 'react';
import { Button, Space, Switch, Table } from 'antd';
import type { TableColumnsType, TableProps, TablePaginationConfig } from 'antd';
import { UnlockOutlined, LockOutlined } from "@ant-design/icons"
import axios from 'axios';

type TableRowSelection<T> = TableProps<T>['rowSelection'];

interface DataType {
  id: number,
  file_name: string,
  file_type: String,
  file_size: number,
  last_access_time: string,
  last_modified_time: string,
  creation_date: string,
}

const buttonStyle: React.CSSProperties = {
  width: 80,
  height: 35,
  display: "flex",
  justifyContent: "center",
  alignItems: "center"
}

const switchStyle: React.CSSProperties = {
  width: 55,
  height: 35,
  display: "flex",
  justifyContent: "center",
  alignItems: "center"
}

const columns: TableColumnsType<DataType> = [
  {
    title: 'File Name',
    dataIndex: 'file_name',
  },
  {
    title: 'File Type',
    dataIndex: 'file_type',
  },
  {
    title: 'Size',
    dataIndex: 'file_size',
  },
  {
    title: 'Last Accessed Time',
    dataIndex: 'last_access_time',
  },
  {
    title: 'Last Modified Time',
    dataIndex: 'last_modified_time',
  },
  {
    title: 'Creation date',
    dataIndex: 'creation_date',
  },
];

const paginationProps: TablePaginationConfig = {
  position: ['bottomCenter'],
  pageSize: 5,
  showSizeChanger: true,
  showQuickJumper: true,
}

const FileManage: React.FC<{ requestUrl: string }> = (props) => {
    const { requestUrl } = props;
    const [selectedRowKeys, setSelectedRowKeys] = useState<React.Key[]>([]);
    const [lockButtons, setLockButtons] = useState(true);
    const [file_list, setFileList] = useState<DataType[]>([]);

    useEffect(() => {
      axios.get("/admin/model_manage",{
        params: {
          useremail: sessionStorage.getItem("useremail"),
          request_dir: requestUrl
        }
      }).then((res) => {
        if (res.status === 200) {
          let files: DataType[] = [];
          for (let idx in res.data) {
            let file = res.data[idx];
            file.id = idx;
            files.push(file);
          }
          setFileList(files);
        }
      }).catch((err) => {
        console.log("get files error: ", err)
      });
    }, []);

    const onSelectChange = (newSelectedRowKeys: React.Key[]) => {
      setSelectedRowKeys(newSelectedRowKeys);
    };

    const handleSwich = () => {
        setLockButtons(!lockButtons);
    }

    const rowSelection: TableRowSelection<DataType> = {
        selectedRowKeys,
        onChange: onSelectChange,
        selections: [
            Table.SELECTION_ALL,
            Table.SELECTION_INVERT,
            Table.SELECTION_NONE,
            {
                key: 'odd',
                text: 'Select Odd Row',
                onSelect: (changeableRowKeys) => {
                let newSelectedRowKeys = [];
                newSelectedRowKeys = changeableRowKeys.filter((_, index) => {
                    if (index % 2 !== 0) {
                    return false;
                    }
                    return true;
                });
                setSelectedRowKeys(newSelectedRowKeys);
                },
            },
            {
                key: 'even',
                text: 'Select Even Row',
                onSelect: (changeableRowKeys) => {
                let newSelectedRowKeys = [];
                newSelectedRowKeys = changeableRowKeys.filter((_, index) => {
                    if (index % 2 !== 0) {
                    return true;
                    }
                    return false;
                });
                setSelectedRowKeys(newSelectedRowKeys);
                },
            },
        ],
    };

    const handleBackupFiles = () => {
      let files2operate: any[] = [];
      for (let idx in selectedRowKeys){
        let key: any = selectedRowKeys[idx];
        files2operate.push(file_list[key].file_name);
      }
      axios.post("/admin/model_manage", {
        useremail: sessionStorage.getItem('useremail'),
        operation_type: "backup",
        files2operate: JSON.stringify(files2operate)
      })
    }

    const handleDeleteFile = () => {
      let files2operate = [];
      for (let idx in selectedRowKeys){
        let key: any = selectedRowKeys[idx];
        files2operate.push(file_list[key].file_name);
      }
      console.log(files2operate);
      axios.post("/admin/model_manage", {
        useremail: sessionStorage.getItem('useremail'),
        operation_type: "remove",
        files2operate: JSON.stringify(files2operate)
      })
    }

    return (
        <>
          <Space align="center" style={{ float: "right" }}>
            <Switch
              checkedChildren={<LockOutlined />}
              unCheckedChildren={<UnlockOutlined />}
              checked={lockButtons}
              onChange={handleSwich}
            />
            <Button type="primary" size="middle" onClick={handleBackupFiles}>
                Back up
            </Button>
            <Button danger type="primary" disabled={lockButtons} size="middle" onClick={handleDeleteFile}>
                Delete
            </Button>
          </Space>
          <Table
            rowKey={item=>item.id}
            rowSelection={rowSelection}
            columns={columns}
            dataSource={file_list}
            pagination={paginationProps}
          />
        </>
    );
};

export default FileManage;